use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to read config file: {0}")]
    ConfigIo(#[from] std::io::Error),

    #[error("failed to parse config file: {0}")]
    ConfigParse(#[from] toml::de::Error),
}
