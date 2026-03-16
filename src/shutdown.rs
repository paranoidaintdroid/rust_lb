use tokio::time::Duration;
use tokio_util::sync::CancellationToken;

pub fn spawn_shutdown_timer(token: CancellationToken, secs: u64) {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(secs)).await;
        token.cancel();
    });
}
