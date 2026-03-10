use tokio::time::{Duration, sleep};
use tokio_util::sync::CancellationToken;

mod config;
mod error;
mod proxy;
mod rate_limit;

use config::load_config;
use error::Error;
use proxy::run;

fn main() -> Result<(), Error> {
    let config = load_config()?;

    println!("Listening on: {}", config.listen_addr);
    println!("Backend count: {}", config.backends.len());

    let token = CancellationToken::new();
    let token_clone = token.clone();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            tokio::spawn(async move {
                sleep(Duration::from_secs(300)).await;
                token.cancel();
            });
            let _ = run(config, token_clone).await;
        });

    Ok(())
}
