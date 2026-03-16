use thiserror::Error;
/// # Error
///
/// The single error type for the whole project.
/// Every module returns `Result<_, Error>` : config, http, proxy, all of it.
///
/// ## Why though?
///
/// Instead of each module having its own error type, we funnel everything
/// into one place. This means `?` just works everywhere, no manual
/// conversion needed.
///
/// ## How though?
///
/// fs::read_to_string()?  ->  io::Error  ->  From<io::Error>  ->  ConfigIo
/// The `#[from]` attribute on a variant auto-generates that `From` impl.
/// and the `?` handles it silently.
#[derive(Debug, Error)]
pub enum Error {
    /// I/O error reading the config file from disk.
    #[error("failed to read config file: {0}")]
    ConfigIo(#[from] std::io::Error),

    /// TOML parse error in the config file.
    #[error("failed to parse config file: {0}")]
    ConfigParse(#[from] toml::de::Error),

    /// General-purpose error for cases without a dedicated variant.
    #[error("{0}")]
    Other(String),

    /// HTTP/1.1 parse error from httparse.
    #[error("http parse error: {0}")]
    HttpParse(#[from] httparse::Error),
}
