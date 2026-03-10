use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{Arc, Mutex},
};

use tokio::{
    io::{copy_bidirectional},
    net::{TcpListener, TcpStream},
    select,
    time::{Duration, sleep, timeout},
};

use tokio_util::sync::CancellationToken;

use crate::config::Config;
use crate::error::Error;
use crate::rate_limit::TokenBucket;

const CONNECTION_TIMEOUT: Duration = Duration::from_secs(60);
const CAPACITY: f64 = 10.0;
const REFILL_RATE: f64 = 2.0;

pub async fn run(config: Config, token: CancellationToken) -> Result<(), Error> {
    // When a TCP connection is closed, the sockect isnt dropped immediately.
    // It goes dormant for 4 minutes (2 * MSL), so when you try to restart it
    // you fail (EADDRINUSE).
    // But tokio automatically applies SO_REUSEADDR, so it allows us to restart
    // without any issues.
    let listener = TcpListener::bind(config.listen_addr).await?;
    let rate_limits: Arc<Mutex<HashMap<IpAddr, TokenBucket>>> =
        Arc::new(Mutex::new(HashMap::new()));

    loop {
        select! {
            result = listener.accept() => {
                let (socket, addr) = match result {
                    Ok(val) => val,
                    Err(e) => {
                        eprintln!("Accept Error {}", e); // TODO : replace with tracing
                        sleep(Duration::from_millis(1)).await;
                        continue;
                    }
                };



                let allowed = {
                    let mut map = rate_limits.lock().expect("the rate limit map was poisoned, this should not have happened");
                    map.entry(addr.ip())
                        .or_insert_with(|| TokenBucket::new(CAPACITY, REFILL_RATE))
                        .allow()
                };

                if !allowed {
                    eprintln!("Rate limited: {addr}"); // TODO : replace with tracing
                    continue;
                }

                tokio::spawn(async move {
                    handle_connection(socket, "127.0.0.1:9000").await;
                });
            }

            _ = token.cancelled() => {
                println!("Shutting down the accept loop"); // TODO : replace with tracing
                return Ok(());
            }
        }
    }
}

async fn handle_connection(mut client: TcpStream, backend_addr: &str) {

    let backend = match timeout(
        Duration::from_secs(5),
        TcpStream::connect(backend_addr)
    ).await {
        Ok(Ok(stream)) => stream,
        Ok(Err(e)) => {
            eprintln!("Backend connection error: {e}"); // TODO : replace with tracing
            return;
        }
        Err(_) => {
            eprintln!("Backend connection timeout"); // TODO : replace with tracing
            return;
        }
    };

    let mut backend = backend;

    select! {

        result = copy_bidirectional(&mut client, &mut backend) => {
            match result {
                Ok((from_client, from_backend)) => {
                    // TODO Step 1.7: tracing
                    // log how many bytes flowed each direction
                    println!(
                        "Connection closed. client->backend: {} bytes, backend->client: {} bytes",
                        from_client, from_backend
                    );
                }

                Err(e) => {
                    eprintln!("Proxy error: {e}"); // TODO : replace with tracing
                }
            }
        }

        _ = sleep(CONNECTION_TIMEOUT) => {
            eprintln!("Connection timed out"); // TODO : replace with tracing
        }
    }
}
