use rust_lb::config::load_config;
use rust_lb::error::Error;
use rust_lb::proxy::run;
use rust_lb::shutdown::spawn_shutdown_timer;
use tokio_util::sync::CancellationToken;

fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let config = load_config()?;

    tracing::info!(
        addr = %config.listen_addr,
        backends = config.backends.len(),
        "starting rust-beast"
    );

    let token = CancellationToken::new();
    let token_clone = token.clone();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build Tokio runtime")
        .block_on(async move {
            spawn_shutdown_timer(token, 300);
            let _ = run(config, token_clone).await;
        });

    Ok(())
}