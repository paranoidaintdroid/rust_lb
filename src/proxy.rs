use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{Arc, Mutex},
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    select,
    time::{Duration, sleep},
};

use tokio_util::sync::CancellationToken;

use crate::config::Config;
use crate::error::Error;
use crate::rate_limit::TokenBucket;

const IDLE_TIMEOUT: Duration = Duration::from_secs(10);
const CAPACITY: f64 = 10.0;
const REFILL_RATE: f64 = 2.0;

pub async fn run(config: Config, token: CancellationToken) -> Result<(), Error> {
    let listener = TcpListener::bind(config.listen_addr).await?;
    let map: Arc<Mutex<HashMap<IpAddr, TokenBucket>>> =
        Arc::new(Mutex::new(HashMap::new()));

    loop {
        select! {
            result = listener.accept() => {
                let (socket, addr) = match result {
                    Ok(val) => val,
                    Err(e) => {
                        eprintln!("Accept Error {}", e);
                        continue;
                    }
                };

                let allowed = {
                    let mut map = map.lock().unwrap();
                    map.entry(addr.ip())
                        .or_insert_with(|| TokenBucket::new(CAPACITY, REFILL_RATE))
                        .allow()
                };

                if !allowed {
                    eprintln!("Rate limited: {addr}");
                    continue;
                }

                tokio::spawn(async move {
                    handle_connection(socket).await;
                });
            }

            _ = token.cancelled() => {
                println!("Shutting down the accept loop");
                return Ok(());
            }
        }
    }
}

async fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0u8; 4096];
    loop {
        select! {
            result = stream.read(&mut buffer) => {
                let n = match result {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("read error: {e}");
                        break;
                    }
                };
                if let Err(e) = stream.write_all(&buffer[..n]).await {
                    eprintln!("write error: {e}");
                    break;
                }
            }
            _ = sleep(IDLE_TIMEOUT) => {
                eprintln!("Connection timed out");
                break;
            }
        }
    }
}