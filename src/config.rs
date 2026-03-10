use crate::error::Error;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub listen_addr: String,
    pub backends: Vec<String>,
}

pub fn load_config() -> Result<Config, Error> {
    let contents = fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}
