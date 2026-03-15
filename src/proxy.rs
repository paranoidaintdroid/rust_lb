use std::{
    collections::HashMap,
    io,
    net::IpAddr,
    sync::{Arc, Mutex},
};

use tokio::{
    io::{AsyncWriteExt, copy_bidirectional},
    net::{TcpListener, TcpStream},
    select,
    time::{Duration, sleep, timeout},
};

use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use crate::config::Config;
use crate::error::Error;
use crate::rate_limit::{DEFAULT_CAPACITY, DEFAULT_REFILL_RATE, TokenBucket};
use crate::http::{parse_request};

const CONNECTION_TIMEOUT: Duration = Duration::from_secs(60);
const BACKEND_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);


pub async fn run(config: Config, token: CancellationToken) -> Result<(), Error> {
    let listener = TcpListener::bind(&config.listen_addr).await?;

    let rate_limits: Arc<Mutex<HashMap<IpAddr, TokenBucket>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let backends = Arc::new(config.backends);

    loop {
        select! {
            result = listener.accept() => {
                let (socket, addr) = match result {
                    Ok(val) => val,
                    Err(e) => {
                        error!(error = %e, "accept failed");
                        sleep(Duration::from_millis(1)).await;
                        continue;
                    }
                };

                if let Err(e) = socket.set_nodelay(true) {
                    warn!(error = %e, "failed to set TCP_NODELAY on client socket");
                }


                let allowed = {
                    let mut map = rate_limits.lock().expect("rate limit map poisoned");

                    map.entry(addr.ip())
                        .or_insert_with(|| TokenBucket::new(DEFAULT_CAPACITY, DEFAULT_REFILL_RATE))
                        .allow()
                };

                if !allowed {
                    warn!(peer = %addr, "rate limit exceeded");
                    continue;
                }

                let backend_addr = match backends.first() {
                    Some(addr) => addr.clone(),
                    None => {
                        error!("no backends configured");
                        continue;
                    }
                };

                tokio::spawn(async move {
                    handle_connection(socket, &backend_addr, addr).await;
                });
            }

            _ = token.cancelled() => {
                info!("shutting down accept loop");
                return Ok(());
            }
        }
    }
}

#[tracing::instrument(skip(client, peer_addr), fields(peer = %peer_addr))]
async fn handle_connection(mut client: TcpStream, backend_addr: &str, peer_addr: std::net::SocketAddr) {
    let (request, raw_bytes) = match parse_request(&mut client).await {
        Ok(val) => val,
        Err(e) => {
            error!(error = ?e, "failed to parse request");
            return;
        }
    };

    info!(
        method = %request.method,
        path = %request.path,
        host = ?request.host,
        "incoming request"
    );

    let mut backend = match connect_to_backend(backend_addr).await {
        Ok(stream) => stream,
        Err(e) => {
            error!(error = ?e, backend = %backend_addr, "backend connection failed");
            return;
        }
    };

    if let Err(e) = backend.write_all(&raw_bytes).await {
        error!(error = ?e, "failed forwarding request to backend");
        return;
    }

    select! {
        result = copy_bidirectional(&mut client, &mut backend) => {
            match result {
                Ok((from_client, from_backend)) => {
                    info!(
                        client_to_backend = from_client,
                        backend_to_client = from_backend,
                        "connection closed"
                    );
                }
                Err(e) => {
                    error!(error = ?e, "proxy copy failed");
                }
            }
        }
        _ = sleep(CONNECTION_TIMEOUT) => {
            warn!("connection timeout reached");
        }
    }
}


async fn connect_to_backend(addr: &str) -> Result<TcpStream, Error> {
    let connect_result = timeout(BACKEND_CONNECT_TIMEOUT, TcpStream::connect(addr)).await;

    let stream = match connect_result {
        Ok(Ok(stream)) => stream,

        Ok(Err(e)) => {
            match e.kind() {
                io::ErrorKind::ConnectionRefused => {
                    warn!(backend = %addr, "backend refused connection");
                }
                io::ErrorKind::TimedOut => {
                    warn!(backend = %addr, "backend connection timed out");
                }
                io::ErrorKind::NetworkUnreachable => {
                    error!(backend = %addr, "network unreachable for backend");
                }
                _ => {
                    error!(backend = %addr, error = ?e, "backend connection error");
                }
            }
            return Err(e.into());
        }

        Err(_) => {
            warn!(backend = %addr, timeout = ?BACKEND_CONNECT_TIMEOUT, "backend connect timeout");
            return Err(Error::Other("backend connect timeout".into()));
        }
    };

    if let Err(e) = stream.set_nodelay(true) {
        warn!(error = ?e, "failed to set TCP_NODELAY on backend socket");
    }

    Ok(stream)
}