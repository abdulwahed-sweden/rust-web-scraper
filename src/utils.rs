use rand::Rng;
use std::time::Duration;
use tokio::time::sleep;

/// User agent rotation for avoiding detection
pub const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:122.0) Gecko/20100101 Firefox/122.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Safari/605.1.15",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36",
];

pub fn get_random_user_agent() -> &'static str {
    let mut rng = rand::rng();
    let index = rng.random_range(0..USER_AGENTS.len());
    USER_AGENTS[index]
}

/// Rate limiter for polite scraping
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_user_agent() {
        let agent = get_random_user_agent();
        assert!(!agent.is_empty());
        assert!(USER_AGENTS.contains(&agent));
    }

    #[test]
    fn test_rate_limiter_creation() {
        let limiter = RateLimiter::new(5.0);
        assert_eq!(limiter.delay_ms, 200);

        let default_limiter = RateLimiter::default();
        assert_eq!(default_limiter.delay_ms, 500);
    }
}
