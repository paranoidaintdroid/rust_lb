use serde::Deserialize;
use std::fs;
use thiserror::Error;

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

fn main() -> Result<(), Error> {
    let config = load_config()?;

    println!("Listening on: {}", config.listen_addr);
    println!("Backend count: {}", config.backends.len());

    Ok(())
}