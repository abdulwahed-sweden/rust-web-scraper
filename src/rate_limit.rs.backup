use std::time::Duration;
use tokio::time::sleep;

pub struct RateLimiter {
    delay_ms: u64,
}

impl RateLimiter {
    pub fn new(requests_per_second: f64) -> Self {
        let delay_ms = (1000.0 / requests_per_second) as u64;
        Self { delay_ms }
    }

    pub async fn wait(&self) {
        sleep(Duration::from_millis(self.delay_ms)).await;
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(2.0) // 2 requests per second by default
    }
}
