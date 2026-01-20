//! Progress tracking for ETL operations.

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Instant;

/// Progress tracker for ETL pipeline execution.
///
/// Displays real-time progress with:
/// - Records processed count
/// - Success/failure counts
/// - Success rate percentage
/// - Elapsed time and ETA
pub struct ProgressTracker {
    bar: ProgressBar,
    start_time: Instant,
    total: usize,
    processed: usize,
    success: usize,
    failed: usize,
}

impl ProgressTracker {
    /// Creates a new progress tracker for the given total record count.
    ///
    /// # Arguments
    ///
    /// * `total` - Total number of records to process
    pub fn new(total: usize) -> Self {
        let bar = ProgressBar::new(total as u64);

        bar.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) | Success: {msg}"
                )
                .unwrap()
                .progress_chars("#>-"),
        );

        Self {
            bar,
            start_time: Instant::now(),
            total,
            processed: 0,
            success: 0,
            failed: 0,
        }
    }

    /// Increments the success counter.
    pub fn inc_success(&mut self) {
        self.success += 1;
        self.processed += 1;
        self.update_bar();
    }

    /// Increments the failure counter.
    pub fn inc_failure(&mut self) {
        self.failed += 1;
        self.processed += 1;
        self.update_bar();
    }

    /// Records a batch completion with success and failure counts.
    ///
    /// # Arguments
    ///
    /// * `success_count` - Number of successful records in the batch
    /// * `failure_count` - Number of failed records in the batch
    pub fn record_batch(&mut self, success_count: usize, failure_count: usize) {
        self.success += success_count;
        self.failed += failure_count;
        self.processed += success_count + failure_count;
        self.update_bar();
    }

    /// Updates the progress bar with current statistics.
    fn update_bar(&mut self) {
        self.bar.set_position(self.processed as u64);

        let success_rate = if self.processed > 0 {
            (self.success as f64 / self.processed as f64) * 100.0
        } else {
            0.0
        };

        self.bar
            .set_message(format!("{:.1}% | Failed: {}", success_rate, self.failed));
    }

    /// Finalizes the progress tracker and displays completion message.
    pub fn finish(&self) {
        self.bar.finish_with_message(format!(
            "Complete! Success: {}, Failed: {}, Total: {}",
            self.success, self.failed, self.processed
        ));
    }

    /// Finalizes with an error message.
    pub fn finish_with_error(&self, error: &str) {
        self.bar.finish_with_message(format!("Error: {}", error));
    }

    /// Returns the current statistics.
    pub fn stats(&self) -> ProgressStats {
        ProgressStats {
            total: self.total,
            processed: self.processed,
            success: self.success,
            failed: self.failed,
            success_rate: if self.processed > 0 {
                (self.success as f64 / self.processed as f64) * 100.0
            } else {
                0.0
            },
            elapsed_secs: self.start_time.elapsed().as_secs(),
        }
    }
}

/// Statistics snapshot from the progress tracker.
#[derive(Debug, Clone)]
pub struct ProgressStats {
    pub total: usize,
    pub processed: usize,
    pub success: usize,
    pub failed: usize,
    pub success_rate: f64,
    pub elapsed_secs: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_tracker_creation() {
        let tracker = ProgressTracker::new(100);
        assert_eq!(tracker.total, 100);
        assert_eq!(tracker.processed, 0);
        assert_eq!(tracker.success, 0);
        assert_eq!(tracker.failed, 0);
    }

    #[test]
    fn test_progress_tracker_inc_success() {
        let mut tracker = ProgressTracker::new(100);
        tracker.inc_success();
        tracker.inc_success();

        assert_eq!(tracker.processed, 2);
        assert_eq!(tracker.success, 2);
        assert_eq!(tracker.failed, 0);
    }

    #[test]
    fn test_progress_tracker_inc_failure() {
        let mut tracker = ProgressTracker::new(100);
        tracker.inc_failure();

        assert_eq!(tracker.processed, 1);
        assert_eq!(tracker.success, 0);
        assert_eq!(tracker.failed, 1);
    }

    #[test]
    fn test_progress_tracker_record_batch() {
        let mut tracker = ProgressTracker::new(100);
        tracker.record_batch(8, 2);

        assert_eq!(tracker.processed, 10);
        assert_eq!(tracker.success, 8);
        assert_eq!(tracker.failed, 2);
    }

    #[test]
    fn test_progress_tracker_stats() {
        let mut tracker = ProgressTracker::new(100);
        tracker.record_batch(75, 5);

        let stats = tracker.stats();
        assert_eq!(stats.total, 100);
        assert_eq!(stats.processed, 80);
        assert_eq!(stats.success, 75);
        assert_eq!(stats.failed, 5);
        assert_eq!(stats.success_rate, 93.75);
    }

    #[test]
    fn test_progress_tracker_success_rate_zero_division() {
        let tracker = ProgressTracker::new(100);
        let stats = tracker.stats();

        assert_eq!(stats.success_rate, 0.0);
    }
}
