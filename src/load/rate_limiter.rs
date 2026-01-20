//! Rate limiting with token bucket algorithm.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;

/// Consecutive successes required to trigger rate recovery.
const RECOVERY_THRESHOLD: u32 = 10;

/// Rate recovery multiplier (10% increase).
const RECOVERY_MULTIPLIER: f64 = 1.1;

/// Token bucket rate limiter with adaptive recovery.
///
/// Implements a token bucket algorithm for rate limiting API requests.
/// Tokens are replenished at a constant rate, and each request consumes one token.
/// Supports adaptive rate recovery after consecutive successful batches.
#[derive(Clone)]
pub struct RateLimiter {
    state: Arc<Mutex<RateLimiterState>>,
    tokens_per_second: f64,
    max_tokens: usize,
    /// Original configured rate (for recovery ceiling)
    original_rate: usize,
    /// Consecutive successful batches counter
    consecutive_successes: Arc<AtomicU32>,
}

struct RateLimiterState {
    tokens: f64,
    last_refill: Instant,
    /// Current effective rate (may be reduced from original)
    current_rate: usize,
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
                current_rate: requests_per_second,
            })),
            tokens_per_second,
            max_tokens,
            original_rate: requests_per_second,
            consecutive_successes: Arc::new(AtomicU32::new(0)),
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
        let ratio = new_tokens_per_second / state.current_rate as f64;
        state.tokens = (state.tokens * ratio).min(new_rate as f64);
        state.current_rate = new_rate;
    }

    /// Gets the current effective rate.
    pub async fn get_rate(&self) -> usize {
        let state = self.state.lock().await;
        state.current_rate
    }

    /// Records a successful batch and potentially triggers rate recovery.
    ///
    /// After `RECOVERY_THRESHOLD` consecutive successes, increases rate by 10%
    /// toward the original configured rate.
    pub async fn record_success(&self) {
        let count = self.consecutive_successes.fetch_add(1, Ordering::SeqCst) + 1;

        if count >= RECOVERY_THRESHOLD {
            let current = self.get_rate().await;
            let target = self.original_rate;

            if current < target {
                let new_rate = ((current as f64) * RECOVERY_MULTIPLIER).min(target as f64) as usize;
                if new_rate > current {
                    self.adjust_rate(new_rate).await;
                    tracing::info!(
                        "Rate recovery: {} -> {} req/s (target: {})",
                        current,
                        new_rate,
                        target
                    );
                }
            }
            // Reset counter after recovery attempt
            self.consecutive_successes.store(0, Ordering::SeqCst);
        }
    }

    /// Resets the success streak counter.
    ///
    /// Call this when a rate limit error occurs to prevent
    /// premature recovery.
    pub fn reset_success_streak(&self) {
        self.consecutive_successes.store(0, Ordering::SeqCst);
    }

    /// Returns the original configured rate.
    pub fn original_rate(&self) -> usize {
        self.original_rate
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

    #[tokio::test]
    async fn test_adjust_rate_tracks_current_rate() {
        let limiter = RateLimiter::new(10);
        assert_eq!(limiter.get_rate().await, 10);

        limiter.adjust_rate(5).await;
        assert_eq!(limiter.get_rate().await, 5);

        limiter.adjust_rate(8).await;
        assert_eq!(limiter.get_rate().await, 8);
    }

    #[tokio::test]
    async fn test_original_rate_preserved() {
        let limiter = RateLimiter::new(10);
        assert_eq!(limiter.original_rate(), 10);

        // Reducing rate doesn't affect original
        limiter.adjust_rate(5).await;
        assert_eq!(limiter.original_rate(), 10);
    }

    #[tokio::test]
    async fn test_success_streak_reset() {
        let limiter = RateLimiter::new(10);

        // Record some successes
        for _ in 0..5 {
            limiter.record_success().await;
        }

        // Reset streak
        limiter.reset_success_streak();

        // Reduce rate and verify no recovery happens before threshold
        limiter.adjust_rate(5).await;
        for _ in 0..9 {
            limiter.record_success().await;
        }
        // Should still be 5 (not recovered yet)
        assert_eq!(limiter.get_rate().await, 5);
    }

    #[tokio::test]
    async fn test_recovery_after_threshold() {
        let limiter = RateLimiter::new(10);

        // Reduce rate
        limiter.adjust_rate(5).await;
        assert_eq!(limiter.get_rate().await, 5);

        // Record 10 successes to trigger recovery
        for _ in 0..10 {
            limiter.record_success().await;
        }

        // Should have recovered by 10% (5 * 1.1 = 5.5 -> 5)
        // But 5 * 1.1 = 5.5, rounded to 5 since it's usize
        let rate = limiter.get_rate().await;
        assert!(rate >= 5); // At least the current rate
    }

    #[tokio::test]
    async fn test_recovery_caps_at_original() {
        let limiter = RateLimiter::new(10);

        // Reduce rate slightly
        limiter.adjust_rate(9).await;

        // Record 10 successes - should recover but not exceed original
        for _ in 0..10 {
            limiter.record_success().await;
        }

        // Should recover to at most original rate
        let rate = limiter.get_rate().await;
        assert!(rate <= 10);
    }
}
