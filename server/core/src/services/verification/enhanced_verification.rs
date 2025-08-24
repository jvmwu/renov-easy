//! Enhanced verification service with advanced security features
//!
//! This module implements:
//! - Progressive delay for failed attempts
//! - Account locking after max attempts
//! - Brute force detection and prevention
//! - Security event logging

use chrono::{DateTime, Duration, Utc};
use tokio::time::sleep;
use tracing;

use crate::domain::entities::verification_code::MAX_ATTEMPTS;
use crate::errors::DomainResult;

use super::types::VerifyCodeResult;

/// Account lock information
#[derive(Debug, Clone)]
pub struct AccountLockInfo {
    /// Phone number that's locked
    pub phone: String,
    /// When the account was locked
    pub locked_at: DateTime<Utc>,
    /// When the lock expires
    pub lock_expires_at: DateTime<Utc>,
    /// Number of consecutive failed attempts
    pub failed_attempts: u32,
    /// Reason for locking
    pub lock_reason: LockReason,
}

/// Reasons for account locking
#[derive(Debug, Clone, PartialEq)]
pub enum LockReason {
    /// Too many failed OTP attempts
    MaxOtpAttemptsExceeded,
    /// Suspicious activity detected
    BruteForceDetected,
    /// Manual lock by admin
    ManualLock,
}

/// Enhanced verification service with security features
pub struct EnhancedVerificationService {
    /// Lock duration after max attempts (in minutes)
    lock_duration_minutes: i64,
    /// Enable progressive delay
    enable_progressive_delay: bool,
    /// Base delay for failed attempts (in milliseconds)
    base_delay_ms: u64,
    /// Maximum delay for failed attempts (in milliseconds)
    max_delay_ms: u64,
}

impl EnhancedVerificationService {
    /// Create a new enhanced verification service
    pub fn new(
        lock_duration_minutes: i64,
        enable_progressive_delay: bool,
        base_delay_ms: u64,
        max_delay_ms: u64,
    ) -> Self {
        Self {
            lock_duration_minutes,
            enable_progressive_delay,
            base_delay_ms,
            max_delay_ms,
        }
    }

    /// Check if an account is locked
    pub async fn is_account_locked(&self, phone: &str) -> DomainResult<Option<AccountLockInfo>> {
        // In production, this would check Redis or database for lock status
        // For now, we'll implement the logic structure

        tracing::debug!(
            phone = phone,
            event = "check_account_lock",
            "Checking if account is locked"
        );

        // This would be retrieved from cache/database
        Ok(None) // Placeholder - would check actual storage
    }

    /// Lock an account due to security violation
    pub async fn lock_account(
        &self,
        phone: &str,
        reason: LockReason,
        failed_attempts: u32,
    ) -> DomainResult<AccountLockInfo> {
        let now = Utc::now();
        let lock_expires_at = now + Duration::minutes(self.lock_duration_minutes);

        let lock_info = AccountLockInfo {
            phone: phone.to_string(),
            locked_at: now,
            lock_expires_at,
            failed_attempts,
            lock_reason: reason.clone(),
        };

        // Log security event
        tracing::warn!(
            phone = phone,
            event = "account_locked",
            reason = ?reason,
            failed_attempts = failed_attempts,
            lock_duration_minutes = self.lock_duration_minutes,
            lock_expires_at = %lock_expires_at,
            "Account locked due to security violation"
        );

        // In production, store lock info in Redis/database
        // self.cache_service.store_account_lock(&lock_info).await?;

        Ok(lock_info)
    }

    /// Unlock an account
    pub async fn unlock_account(&self, phone: &str) -> DomainResult<()> {
        tracing::info!(
            phone = phone,
            event = "account_unlocked",
            "Account unlocked"
        );

        // In production, remove lock from Redis/database
        // self.cache_service.remove_account_lock(phone).await?;

        Ok(())
    }

    /// Calculate progressive delay based on attempt number
    pub fn calculate_delay(&self, attempt_number: u32) -> u64 {
        if !self.enable_progressive_delay {
            return 0;
        }

        // Exponential backoff: delay = base_delay * 2^(attempt_number - 1)
        let delay = self.base_delay_ms * 2u64.pow(attempt_number.saturating_sub(1));

        // Cap at maximum delay
        delay.min(self.max_delay_ms)
    }

    /// Apply progressive delay for failed attempts
    pub async fn apply_progressive_delay(&self, attempt_number: u32) {
        if !self.enable_progressive_delay {
            return;
        }

        let delay_ms = self.calculate_delay(attempt_number);
        if delay_ms > 0 {
            tracing::debug!(
                event = "progressive_delay",
                attempt = attempt_number,
                delay_ms = delay_ms,
                "Applying progressive delay for failed attempt"
            );

            sleep(std::time::Duration::from_millis(delay_ms)).await;
        }
    }

    /// Detect potential brute force attack
    pub async fn detect_brute_force(&self, phone: &str, timeframe_minutes: i64) -> DomainResult<bool> {
        // Check for patterns indicating brute force:
        // 1. Multiple attempts from same phone in short time
        // 2. Attempts with sequential codes
        // 3. Unusual geographic patterns (if IP tracking is enabled)

        tracing::debug!(
            phone = phone,
            event = "brute_force_check",
            timeframe_minutes = timeframe_minutes,
            "Checking for brute force patterns"
        );

        // In production, this would analyze attempt patterns from logs/cache
        // For now, return the logic structure

        Ok(false) // Placeholder - would check actual patterns
    }

    /// Enhanced verify code with security features
    pub async fn verify_code_with_security(
        &self,
        phone: &str,
        code: &str,
        current_attempts: u32,
    ) -> DomainResult<VerifyCodeResult> {
        // Check if account is locked
        if let Some(lock_info) = self.is_account_locked(phone).await? {
            if lock_info.lock_expires_at > Utc::now() {
                let minutes_remaining = (lock_info.lock_expires_at - Utc::now()).num_minutes();

                tracing::warn!(
                    phone = phone,
                    event = "verification_blocked_locked",
                    lock_reason = ?lock_info.lock_reason,
                    minutes_remaining = minutes_remaining,
                    "Verification attempt blocked - account locked"
                );

                return Ok(VerifyCodeResult {
                    success: false,
                    remaining_attempts: Some(0),
                    error_message: Some(format!(
                        "Account locked. Try again in {} minutes",
                        minutes_remaining.max(1)
                    )),
                });
            } else {
                // Lock expired, unlock account
                self.unlock_account(phone).await?;
            }
        }

        // Check for brute force patterns
        if self.detect_brute_force(phone, 5).await? {
            // Lock account due to brute force detection
            self.lock_account(phone, LockReason::BruteForceDetected, current_attempts).await?;

            tracing::error!(
                phone = phone,
                event = "brute_force_detected",
                "Brute force attack detected - account locked"
            );

            return Ok(VerifyCodeResult {
                success: false,
                remaining_attempts: Some(0),
                error_message: Some("Suspicious activity detected. Account temporarily locked.".to_string()),
            });
        }

        // Apply progressive delay if this is not the first attempt
        if current_attempts > 0 {
            self.apply_progressive_delay(current_attempts).await;
        }

        // The actual verification would happen here through the cache service
        // This is just the security wrapper

        Ok(VerifyCodeResult {
            success: false,
            remaining_attempts: Some((MAX_ATTEMPTS as u32).saturating_sub(current_attempts) as i32),
            error_message: None,
        })
    }

    /// Handle failed verification attempt
    pub async fn handle_failed_attempt(
        &self,
        phone: &str,
        current_attempts: u32,
    ) -> DomainResult<()> {
        // Check if max attempts exceeded
        if current_attempts >= MAX_ATTEMPTS as u32 {
            // Lock the account
            self.lock_account(phone, LockReason::MaxOtpAttemptsExceeded, current_attempts).await?;

            tracing::error!(
                phone = phone,
                event = "max_attempts_exceeded",
                attempts = current_attempts,
                "Maximum OTP attempts exceeded - account locked"
            );
        }

        Ok(())
    }

    /// Log security event for audit trail
    pub fn log_security_event(&self, event_type: &str, phone: &str, details: &str) {
        tracing::info!(
            event = "security_audit",
            event_type = event_type,
            phone = phone,
            details = details,
            timestamp = %Utc::now(),
            "Security event logged for audit"
        );
    }

    /// Get verification statistics for monitoring
    pub async fn get_verification_stats(&self, phone: &str) -> DomainResult<VerificationStats> {
        // In production, this would query metrics from monitoring system
        Ok(VerificationStats {
            total_attempts: 0,
            successful_verifications: 0,
            failed_verifications: 0,
            account_locks: 0,
            last_attempt: None,
            last_successful: None,
        })
    }
}

/// Verification statistics for monitoring
#[derive(Debug, Clone)]
pub struct VerificationStats {
    pub total_attempts: u64,
    pub successful_verifications: u64,
    pub failed_verifications: u64,
    pub account_locks: u64,
    pub last_attempt: Option<DateTime<Utc>>,
    pub last_successful: Option<DateTime<Utc>>,
}
