use serde::Deserialize;
use std::{
    fs,
    sync::{Arc, Mutex},
};
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    select,
    time::{Duration, sleep},
};

const IDLE_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Deserialize)]
struct Config {
    listen_addr: String,
    backends: Vec<String>,
}

#[derive(Debug, Error)]
enum Error {
    #[error("failed to read config file: {0}")]
    ConfigIo(#[from] std::io::Error),

    #[error("failed to parse config file: {0}")]
    ConfigParse(#[from] toml::de::Error),
}

fn load_config() -> Result<Config, Error> {
    let contents = fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}

async fn run(config: Config) -> Result<(), Error> {
    let listener = TcpListener::bind(config.listen_addr).await?;
    let count = Arc::new(Mutex::new(0));
    loop {
        let (socket, _addr) = match listener.accept().await {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Accept Error {}", e);
                continue;
            }
        };

        let count_clone = Arc::clone(&count);

        tokio::spawn(async move {
            let count = counted_echo(socket, count_clone).await;
            println!("connection closed after {} chunks", count);
        });
    }
}

async fn counted_echo(mut stream: TcpStream, count_clone: Arc<Mutex<u64>>) -> u64 {
    let mut buffer = [0u8; 4096];
    loop {
        select! {
            result = stream.read(&mut buffer) =>{
                let n = match result {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(e) =>{
                        eprintln!("read error : {e}");
                        break;
                    },
                };

                {
                    let mut count = count_clone.lock().unwrap();
                    *count += 1;
                }
                if let Err(e) = stream.write_all(&buffer[..n]).await {
                    eprintln!("write error: {e}");
                    break;
                } 
            }

            _ = sleep(IDLE_TIMEOUT) =>{
                eprintln!{"Connection time out"};
                let count = count_clone.lock().unwrap();
                return *count;
            }
        }
    }
    let count = count_clone.lock().unwrap();
    *count
}

fn main() -> Result<(), Error> {
    let config = load_config()?;

    println!("Listening on: {}", config.listen_addr);
    println!("Backend count: {}", config.backends.len());

    let _ = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(run(config));

    Ok(())
}
