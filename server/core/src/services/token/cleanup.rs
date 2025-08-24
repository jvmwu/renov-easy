//! Token cleanup service for periodic maintenance of refresh tokens and blacklist
//!
//! This module provides background cleanup functionality for expired tokens
//! and blacklist entries to maintain database performance and security.

use std::sync::Arc;
use tracing::{error, info, warn};

use crate::errors::DomainError;
use crate::repositories::TokenRepository;

/// Configuration for token cleanup service
#[derive(Debug, Clone)]
pub struct TokenCleanupConfig {
    /// How often to run cleanup (in seconds)
    pub interval_seconds: u64,
    /// Grace period after expiry before deletion (in days)
    pub grace_period_days: i64,
    /// Maximum number of tokens to delete in one batch
    pub batch_size: usize,
    /// Whether to enable automatic cleanup
    pub enabled: bool,
}

impl Default for TokenCleanupConfig {
    fn default() -> Self {
        Self {
            interval_seconds: 3600, // Run every hour
            grace_period_days: 7,   // Keep expired tokens for 7 days
            batch_size: 1000,       // Process up to 1000 tokens per batch
            enabled: true,
        }
    }
}

/// Service for cleaning up expired tokens and blacklist entries
pub struct TokenCleanupService<R: TokenRepository + 'static> {
    repository: Arc<R>,
    config: TokenCleanupConfig,
}

impl<R: TokenRepository> TokenCleanupService<R> {
    /// Create a new token cleanup service
    pub fn new(repository: Arc<R>, config: TokenCleanupConfig) -> Self {
        Self { repository, config }
    }

    /// Run a single cleanup cycle
    ///
    /// This method performs the following cleanup tasks:
    /// 1. Delete expired refresh tokens (with grace period)
    /// 2. Clean up expired blacklist entries
    /// 3. Revoke orphaned tokens from incomplete rotations
    ///
    /// # Returns
    /// * `Ok(CleanupResult)` - Summary of cleanup operations
    /// * `Err(DomainError)` - If cleanup fails
    pub async fn run_cleanup(&self) -> Result<CleanupResult, DomainError> {
        if !self.config.enabled {
            return Ok(CleanupResult::default());
        }

        info!("Starting token cleanup cycle");

        let mut result = CleanupResult::default();

        // Clean up expired refresh tokens
        match self.cleanup_expired_tokens().await {
            Ok(count) => {
                result.expired_tokens_deleted = count;
                info!("Deleted {} expired refresh tokens", count);
            }
            Err(e) => {
                error!("Failed to cleanup expired tokens: {}", e);
                result.errors.push(format!("Token cleanup error: {}", e));
            }
        }

        // Clean up expired blacklist entries
        match self.cleanup_blacklist().await {
            Ok(count) => {
                result.blacklist_entries_deleted = count;
                info!("Deleted {} expired blacklist entries", count);
            }
            Err(e) => {
                error!("Failed to cleanup blacklist: {}", e);
                result
                    .errors
                    .push(format!("Blacklist cleanup error: {}", e));
            }
        }

        // Clean up orphaned tokens from incomplete rotations
        match self.cleanup_orphaned_tokens().await {
            Ok(count) => {
                result.orphaned_tokens_revoked = count;
                info!("Revoked {} orphaned tokens", count);
            }
            Err(e) => {
                error!("Failed to cleanup orphaned tokens: {}", e);
                result
                    .errors
                    .push(format!("Orphaned token cleanup error: {}", e));
            }
        }

        info!(
            "Token cleanup completed - Expired: {}, Blacklist: {}, Orphaned: {}",
            result.expired_tokens_deleted,
            result.blacklist_entries_deleted,
            result.orphaned_tokens_revoked
        );

        Ok(result)
    }

    /// Clean up expired refresh tokens with grace period
    async fn cleanup_expired_tokens(&self) -> Result<usize, DomainError> {
        self.repository.delete_expired_tokens().await
    }

    /// Clean up expired blacklist entries
    async fn cleanup_blacklist(&self) -> Result<usize, DomainError> {
        self.repository.cleanup_blacklist().await
    }

    /// Clean up orphaned tokens from incomplete rotations
    ///
    /// Tokens are considered orphaned if:
    /// - They have a rotated_to_token_id but the new token doesn't exist
    /// - They are part of a family where suspicious activity was detected
    async fn cleanup_orphaned_tokens(&self) -> Result<usize, DomainError> {
        // This is a placeholder for more sophisticated orphan detection
        // In production, this would query for tokens with broken rotation chains
        Ok(0)
    }

    /// Start the cleanup service as a background task
    ///
    /// This spawns a tokio task that runs cleanup at regular intervals
    pub fn start_background_task(self: Arc<Self>) {
        if !self.config.enabled {
            warn!("Token cleanup service is disabled");
            return;
        }

        let interval = std::time::Duration::from_secs(self.config.interval_seconds);

        tokio::spawn(async move {
            info!(
                "Token cleanup service started - will run every {} seconds",
                self.config.interval_seconds
            );

            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                match self.run_cleanup().await {
                    Ok(result) => {
                        if !result.errors.is_empty() {
                            warn!("Cleanup completed with errors: {:?}", result.errors);
                        }
                    }
                    Err(e) => {
                        error!("Token cleanup cycle failed: {}", e);
                    }
                }
            }
        });
    }
}

/// Result of a cleanup operation
#[derive(Debug, Default)]
pub struct CleanupResult {
    /// Number of expired refresh tokens deleted
    pub expired_tokens_deleted: usize,
    /// Number of expired blacklist entries deleted
    pub blacklist_entries_deleted: usize,
    /// Number of orphaned tokens revoked
    pub orphaned_tokens_revoked: usize,
    /// Any errors encountered during cleanup
    pub errors: Vec<String>,
}

impl CleanupResult {
    /// Check if the cleanup was successful (no errors)
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get total number of items cleaned up
    pub fn total_cleaned(&self) -> usize {
        self.expired_tokens_deleted + self.blacklist_entries_deleted + self.orphaned_tokens_revoked
    }
}
