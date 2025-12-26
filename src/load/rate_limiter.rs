//! Rate limiting with token bucket algorithm.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;

/// Token bucket rate limiter.
///
/// Implements a token bucket algorithm for rate limiting API requests.
/// Tokens are replenished at a constant rate, and each request consumes one token.
#[derive(Clone)]
pub struct RateLimiter {
    state: Arc<Mutex<RateLimiterState>>,
    tokens_per_second: f64,
    max_tokens: usize,
}

struct RateLimiterState {
    tokens: f64,
    last_refill: Instant,
}

impl RateLimiter {
    /// Creates a new rate limiter.
    ///
    /// # Arguments
    ///
    /// * `requests_per_second` - Maximum requests per second
    ///
    /// # Example
    ///
    /// ```
    /// use basevn_migro::load::rate_limiter::RateLimiter;
    /// let limiter = RateLimiter::new(10); // 10 requests per second
    /// ```
    pub fn new(requests_per_second: usize) -> Self {
        let tokens_per_second = requests_per_second as f64;
        let max_tokens = requests_per_second;

        Self {
            state: Arc::new(Mutex::new(RateLimiterState {
                tokens: tokens_per_second,
                last_refill: Instant::now(),
            })),
            tokens_per_second,
            max_tokens,
        }
    }

    /// Acquires permission to make a request.
    ///
    /// Blocks until a token is available. Automatically refills tokens
    /// based on time elapsed since last refill.
    pub async fn acquire(&self) {
        loop {
            let mut state = self.state.lock().await;

            // Refill tokens based on time elapsed
            let now = Instant::now();
            let elapsed = now.duration_since(state.last_refill).as_secs_f64();
            let new_tokens = elapsed * self.tokens_per_second;

            state.tokens = (state.tokens + new_tokens).min(self.max_tokens as f64);
            state.last_refill = now;

            // Try to consume a token
            if state.tokens >= 1.0 {
                state.tokens -= 1.0;
                return;
            }

            // Calculate wait time until next token is available
            let tokens_needed = 1.0 - state.tokens;
            let wait_seconds = tokens_needed / self.tokens_per_second;
            let wait_duration = Duration::from_secs_f64(wait_seconds);

            // Release lock and wait
            drop(state);
            sleep(wait_duration).await;
        }
    }

    /// Adjusts the rate limit dynamically.
    ///
    /// Useful for adaptive rate limiting based on server responses (e.g., 429).
    ///
    /// # Arguments
    ///
    /// * `new_rate` - New requests per second
    pub async fn adjust_rate(&self, new_rate: usize) {
        let mut state = self.state.lock().await;
        let new_tokens_per_second = new_rate as f64;

        // Update tokens proportionally
        let ratio = new_tokens_per_second / self.tokens_per_second;
        state.tokens = (state.tokens * ratio).min(new_rate as f64);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Instant;

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new(5); // 5 requests per second

        let start = Instant::now();

        // Should allow 5 requests immediately
        for _ in 0..5 {
            limiter.acquire().await;
        }

        let elapsed = start.elapsed();
        assert!(elapsed < Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_rate_limiter_delays() {
        let limiter = RateLimiter::new(2); // 2 requests per second

        let start = Instant::now();

        // First 2 requests should be immediate
        limiter.acquire().await;
        limiter.acquire().await;

        // Third request should wait ~0.5 seconds
        limiter.acquire().await;

        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(400)); // Allow some margin
        assert!(elapsed < Duration::from_millis(700));
    }
}
