use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{Arc, Mutex},
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, copy_bidirectional},
    net::{TcpListener, TcpStream},
    select,
    time::{Duration, sleep, timeout},
};

use tokio_util::sync::CancellationToken;

use crate::config::Config;
use crate::error::Error;
use crate::rate_limit::{TokenBucket, DEFAULT_CAPACITY, DEFAULT_REFILL_RATE};
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(60);
const MAX_HEADER_SIZE: usize = 8192;

pub struct RequestInfo {
    pub method: String,
    pub path: String,
    pub host: Option<String>,
}

pub async fn run(config: Config, token: CancellationToken) -> Result<(), Error> {
    // When a TCP connection closes the socket lingers in TIME_WAIT.
    // Tokio sets SO_REUSEADDR automatically so restart works safely.
    let listener = TcpListener::bind(config.listen_addr).await?;

    let rate_limits: Arc<Mutex<HashMap<IpAddr, TokenBucket>>> =
        Arc::new(Mutex::new(HashMap::new()));

    loop {
        select! {

            result = listener.accept() => {

                let (socket, addr) = match result {
                    Ok(val) => val,
                    Err(e) => {
                        eprintln!("Accept error: {}", e);
                        sleep(Duration::from_millis(1)).await;
                        continue;
                    }
                };

                if let Err(e) = socket.set_nodelay(true) {
                    eprintln!("set_nodelay failed on client socket: {e}");
                }

                let allowed = {
                    let mut map = rate_limits
                        .lock()
                        .expect("rate limit map poisoned");

                    map.entry(addr.ip())
                        .or_insert_with(|| TokenBucket::new(DEFAULT_CAPACITY, DEFAULT_REFILL_RATE))
                        .allow()
                };

                if !allowed {
                    eprintln!("Rate limited: {addr}");
                    continue;
                }

                tokio::spawn(async move {
                    handle_connection(socket, "127.0.0.1:9000").await;
                });
            }

            _ = token.cancelled() => {
                println!("Shutting down accept loop");
                return Ok(());
            }
        }
    }
}

async fn handle_connection(mut client: TcpStream, backend_addr: &str) {
    let (request, raw_bytes) = match parse_request(&mut client).await {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Failed to parse request: {e}");
            return;
        }
    };

    eprintln!("Incoming request: {} {}", request.method, request.path);

    let backend = match timeout(Duration::from_secs(5), TcpStream::connect(backend_addr)).await {
        Ok(Ok(stream)) => stream,

        Ok(Err(e)) => {
            eprintln!("Backend connection error: {e}");
            return;
        }

        Err(_) => {
            eprintln!("Backend connection timeout");
            return;
        }
    };

    let mut backend = backend;

    if let Err(e) = backend.set_nodelay(true) {
        eprintln!("set_nodelay failed on backend socket: {e}");
    }

    if let Err(e) = backend.write_all(&raw_bytes).await {
        eprintln!("Failed forwarding request: {e}");
        return;
    }

    select! {

        result = copy_bidirectional(&mut client, &mut backend) => {

            match result {

                Ok((from_client, from_backend)) => {

                    println!(
                        "Connection closed. client->backend: {} bytes, backend->client: {} bytes",
                        from_client,
                        from_backend
                    );

                }

                Err(e) => {
                    eprintln!("Proxy error: {e}");
                }

            }
        }

        _ = sleep(CONNECTION_TIMEOUT) => {
            eprintln!("Connection timed out");
        }

    }
}

async fn parse_request(client: &mut TcpStream) -> Result<(RequestInfo, Vec<u8>), Error> {
    // TODO replace with a pooled buffer — 8KB × 500K connections = 4GB
    let mut buffer = vec![0u8; MAX_HEADER_SIZE];
    let mut filled = 0;

    loop {
        let n = client.read(&mut buffer[filled..]).await?;

        if n == 0 {
            return Err(Error::Other("Client closed connection".into()));
        }

        filled += n;

        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut headers);

        match req.parse(&buffer[..filled])? {
            httparse::Status::Complete(_offset) => {
                let method = req.method.unwrap_or("").to_string();
                let path = req.path.unwrap_or("").to_string();

                let host = req
                    .headers
                    .iter()
                    .find(|h| h.name.eq_ignore_ascii_case("host"))
                    .and_then(|h| std::str::from_utf8(h.value).ok())
                    .map(|s| s.to_string());

                let info = RequestInfo { method, path, host };

                return Ok((info, buffer[..filled].to_vec()));
            }

            httparse::Status::Partial => {
                if filled >= MAX_HEADER_SIZE {
                    return Err(Error::Other("Header too large".into()));
                }

                continue;
            }
        }
    }
}
