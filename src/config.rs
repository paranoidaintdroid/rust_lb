use crate::error::Error;
use serde::Deserialize;
use std::fs;

/// Holds the configuration for the proxy server.
///
/// This struct matches the structure of the `config.toml` file.
/// `serde` will automatically fill these fields when we read
/// and deserialize the config file.
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Address where the proxy server should listen.
    pub listen_addr: String,

    /// List of backend servers that the proxy will forward requests to.
    pub backends: Vec<String>,
}

/// Reads the `config.toml` file and converts it into a `Config` struct.
///
/// Whats happening here?
/// 
/// Read the config file as a string.
/// Convert (deserialize) the TOML data into the `Config` struct.
/// Return the config if everything works.
///
/// If something goes wrong (file missing, invalid TOML, etc etc)
/// it returns an `Error`. 
pub fn load_config() -> Result<Config, Error> {
    let contents = fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}