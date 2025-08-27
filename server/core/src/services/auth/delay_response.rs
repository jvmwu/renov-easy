//! Delay response service for progressive authentication delays to prevent brute force attacks

use std::time::Duration;
use tracing::warn;

/// Configuration for delay response service
#[derive(Debug, Clone)]
pub struct DelayResponseConfig {
    /// Base delay in milliseconds for first failed attempt
    pub base_delay_ms: u64,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Maximum delay in milliseconds
    pub max_delay_ms: u64,
    /// Number of attempts before applying delay
    pub delay_after_attempts: u32,
}

impl Default for DelayResponseConfig {
    fn default() -> Self {
        Self {
            base_delay_ms: 500,        // 500ms base delay
            backoff_multiplier: 2.0,    // Double each time
            max_delay_ms: 30000,        // 30 seconds max
            delay_after_attempts: 1,     // Start delay after first failure
        }
    }
}

/// Service for implementing progressive delay responses to prevent brute force attacks
pub struct DelayResponseService {
    config: DelayResponseConfig,
}

impl DelayResponseService {
    /// Create new delay response service with configuration
    pub fn new(config: DelayResponseConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(DelayResponseConfig::default())
    }

    /// Calculate delay based on number of failed attempts
    pub fn calculate_delay(&self, failed_attempts: u32) -> Duration {
        if failed_attempts < self.config.delay_after_attempts {
            return Duration::from_millis(0);
        }

        let attempt_index = (failed_attempts - self.config.delay_after_attempts) as f64;
        let delay_ms = (self.config.base_delay_ms as f64)
            * self.config.backoff_multiplier.powf(attempt_index);

        let capped_delay = delay_ms.min(self.config.max_delay_ms as f64) as u64;

        Duration::from_millis(capped_delay)
    }

    /// Apply delay asynchronously
    pub async fn apply_delay(&self, failed_attempts: u32) {
        let delay = self.calculate_delay(failed_attempts);

        if delay.as_millis() > 0 {
            warn!(
                failed_attempts = failed_attempts,
                delay_ms = delay.as_millis(),
                "Applying progressive delay for failed authentication"
            );
            tokio::time::sleep(delay).await;
        }
    }

    /// Get delay information for logging/metrics
    pub fn get_delay_info(&self, failed_attempts: u32) -> DelayInfo {
        let delay = self.calculate_delay(failed_attempts);
        let next_delay = self.calculate_delay(failed_attempts + 1);

        DelayInfo {
            current_delay_ms: delay.as_millis() as u64,
            next_delay_ms: next_delay.as_millis() as u64,
            failed_attempts,
            is_delayed: delay.as_millis() > 0,
            at_max_delay: delay.as_millis() >= self.config.max_delay_ms as u128,
        }
    }
}

/// Information about current delay state
#[derive(Debug, Clone)]
pub struct DelayInfo {
    pub current_delay_ms: u64,
    pub next_delay_ms: u64,
    pub failed_attempts: u32,
    pub is_delayed: bool,
    pub at_max_delay: bool,
}
