use std::time::Duration;
use tokio::time::sleep;

pub async fn sleep_ms(ms: u64) {
  sleep(Duration::from_millis(ms)).await;
}