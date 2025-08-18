//! Audit service for recording authentication attempts and security events.
//!
//! This service provides asynchronous audit logging capabilities to record
//! authentication attempts, failures, and suspicious activities without
//! blocking the main authentication flow.

use std::sync::Arc;
use chrono::{Duration, Utc};
use tokio::task;
use uuid::Uuid;

use crate::domain::entities::audit::{AuditLog, actions};
use crate::errors::DomainResult;
use crate::repositories::AuditLogRepository;

/// Configuration for the audit service
#[derive(Debug, Clone)]
pub struct AuditServiceConfig {
    /// Time window (in minutes) for checking failed attempts
    pub failed_attempts_window_minutes: i64,
    /// Maximum failed attempts before triggering alerts
    pub max_failed_attempts: usize,
    /// Time window (in minutes) for detecting suspicious activity
    pub suspicious_activity_window_minutes: i64,
    /// Whether to run audit writes asynchronously
    pub async_writes: bool,
}

impl Default for AuditServiceConfig {
    fn default() -> Self {
        Self {
            failed_attempts_window_minutes: 15,
            max_failed_attempts: 5,
            suspicious_activity_window_minutes: 60,
            async_writes: true,
        }
    }
}

/// Service for managing audit logs and security monitoring
pub struct AuditService<R>
where
    R: AuditLogRepository,
{
    repository: Arc<R>,
    config: AuditServiceConfig,
}

impl<R> AuditService<R>
where
    R: AuditLogRepository + 'static,
{
    /// Create a new audit service
    pub fn new(repository: Arc<R>, config: AuditServiceConfig) -> Self {
        Self { repository, config }
    }

    /// Log an authentication attempt
    ///
    /// # Arguments
    /// * `action` - The action being performed (e.g., login, verify_code)
    /// * `success` - Whether the action succeeded
    /// * `user_id` - Optional user ID if available
    /// * `phone_hash` - Optional hashed phone number
    /// * `ip_address` - Optional IP address
    /// * `user_agent` - Optional user agent string
    /// * `error_message` - Optional error message for failures
    pub async fn log_auth_attempt(
        &self,
        action: impl Into<String>,
        success: bool,
        user_id: Option<Uuid>,
        phone_hash: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
        error_message: Option<String>,
    ) -> DomainResult<()> {
        let mut audit_log = AuditLog::new(action, success);

        if let Some(uid) = user_id {
            audit_log = audit_log.with_user(uid);
        }

        if let Some(ph) = phone_hash {
            audit_log = audit_log.with_phone_hash(ph);
        }

        audit_log = audit_log.with_request_context(ip_address, user_agent);

        if let Some(err) = error_message {
            audit_log = audit_log.with_error(err);
        }

        self.write_log(audit_log).await
    }

    /// Log a send code attempt
    pub async fn log_send_code(
        &self,
        phone_hash: &str,
        success: bool,
        ip_address: Option<String>,
        user_agent: Option<String>,
        error_message: Option<String>,
    ) -> DomainResult<()> {
        self.log_auth_attempt(
            actions::SEND_CODE_ATTEMPT,
            success,
            None,
            Some(phone_hash.to_string()),
            ip_address,
            user_agent,
            error_message,
        )
        .await
    }

    /// Log a verify code attempt
    pub async fn log_verify_code(
        &self,
        phone_hash: &str,
        success: bool,
        user_id: Option<Uuid>,
        ip_address: Option<String>,
        user_agent: Option<String>,
        error_message: Option<String>,
    ) -> DomainResult<()> {
        self.log_auth_attempt(
            actions::VERIFY_CODE_ATTEMPT,
            success,
            user_id,
            Some(phone_hash.to_string()),
            ip_address,
            user_agent,
            error_message,
        )
        .await
    }

    /// Log a login attempt
    pub async fn log_login(
        &self,
        user_id: Option<Uuid>,
        phone_hash: Option<String>,
        success: bool,
        ip_address: Option<String>,
        user_agent: Option<String>,
        error_message: Option<String>,
    ) -> DomainResult<()> {
        self.log_auth_attempt(
            actions::LOGIN_ATTEMPT,
            success,
            user_id,
            phone_hash,
            ip_address,
            user_agent,
            error_message,
        )
        .await
    }

    /// Log a rate limit exceeded event
    pub async fn log_rate_limit_exceeded(
        &self,
        phone_hash: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> DomainResult<()> {
        self.log_auth_attempt(
            actions::RATE_LIMIT_EXCEEDED,
            false,
            None,
            phone_hash,
            ip_address,
            user_agent,
            Some("Rate limit exceeded".to_string()),
        )
        .await
    }

    /// Log suspicious activity detection
    pub async fn log_suspicious_activity(
        &self,
        phone_hash: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
        reason: &str,
    ) -> DomainResult<()> {
        self.log_auth_attempt(
            actions::SUSPICIOUS_ACTIVITY,
            false,
            None,
            phone_hash,
            ip_address,
            user_agent,
            Some(reason.to_string()),
        )
        .await
    }

    /// Check if there are too many failed attempts for a given identifier
    ///
    /// # Returns
    /// * `true` if the number of failed attempts exceeds the threshold
    pub async fn check_failed_attempts(
        &self,
        action: &str,
        phone_hash: Option<&str>,
        ip_address: Option<&str>,
    ) -> DomainResult<bool> {
        let since = Utc::now() - Duration::minutes(self.config.failed_attempts_window_minutes);
        
        let count = self
            .repository
            .count_failed_attempts(action, phone_hash, ip_address, since)
            .await?;

        Ok(count >= self.config.max_failed_attempts)
    }

    /// Detect suspicious activity patterns
    ///
    /// # Returns
    /// * `true` if suspicious activity is detected
    pub async fn detect_suspicious_activity(
        &self,
        ip_address: Option<&str>,
    ) -> DomainResult<bool> {
        let since = Utc::now() - Duration::minutes(self.config.suspicious_activity_window_minutes);
        
        let logs = self
            .repository
            .find_suspicious_activity(ip_address, since)
            .await?;

        // Simple heuristic: too many different phone numbers from same IP
        if let Some(_ip) = ip_address {
            let unique_phones: std::collections::HashSet<_> = logs
                .iter()
                .filter_map(|log| log.phone_hash.as_ref())
                .collect();

            if unique_phones.len() > 5 {
                return Ok(true);
            }
        }

        // Too many total failures
        Ok(logs.len() > 10)
    }

    /// Get recent audit logs for a user
    pub async fn get_user_audit_logs(
        &self,
        user_id: Uuid,
        limit: usize,
    ) -> DomainResult<Vec<AuditLog>> {
        self.repository.find_by_user(user_id, limit).await
    }

    /// Get recent audit logs for a phone number
    pub async fn get_phone_audit_logs(
        &self,
        phone_hash: &str,
        limit: usize,
    ) -> DomainResult<Vec<AuditLog>> {
        self.repository.find_by_phone_hash(phone_hash, limit).await
    }

    /// Internal method to write audit logs
    ///
    /// If async_writes is enabled, the write happens in a background task
    /// to avoid blocking the main flow.
    async fn write_log(&self, audit_log: AuditLog) -> DomainResult<()> {
        if self.config.async_writes {
            let repository = Arc::clone(&self.repository);
            
            // Spawn a background task for async write
            task::spawn(async move {
                if let Err(e) = repository.create(&audit_log).await {
                    // Log the error but don't fail the main operation
                    eprintln!("Failed to write audit log: {:?}", e);
                }
            });
            
            Ok(())
        } else {
            // Synchronous write
            self.repository.create(&audit_log).await
        }
    }
}