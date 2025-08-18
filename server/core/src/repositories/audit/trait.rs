//! Audit log repository trait defining the interface for audit log persistence.

use async_trait::async_trait;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::domain::entities::audit::AuditLog;
use crate::errors::DomainError;

/// Repository trait for AuditLog entity persistence operations
///
/// This trait defines the contract for audit log data access operations.
/// Implementations should handle async database writes efficiently to avoid
/// blocking authentication flows.
#[async_trait]
pub trait AuditLogRepository: Send + Sync {
    /// Create a new audit log entry
    ///
    /// # Arguments
    /// * `audit_log` - The audit log entry to persist
    ///
    /// # Returns
    /// * `Ok(())` on successful creation
    /// * `Err(DomainError)` if the operation fails
    async fn create(&self, audit_log: &AuditLog) -> Result<(), DomainError>;

    /// Find audit logs by user ID
    ///
    /// # Arguments
    /// * `user_id` - The user ID to search for
    /// * `limit` - Maximum number of records to return
    ///
    /// # Returns
    /// * List of audit logs for the user, ordered by created_at descending
    async fn find_by_user(
        &self,
        user_id: Uuid,
        limit: usize,
    ) -> Result<Vec<AuditLog>, DomainError>;

    /// Find audit logs by phone hash
    ///
    /// # Arguments
    /// * `phone_hash` - The hashed phone number to search for
    /// * `limit` - Maximum number of records to return
    ///
    /// # Returns
    /// * List of audit logs for the phone number, ordered by created_at descending
    async fn find_by_phone_hash(
        &self,
        phone_hash: &str,
        limit: usize,
    ) -> Result<Vec<AuditLog>, DomainError>;

    /// Find recent failed attempts for a given action and identifier
    ///
    /// # Arguments
    /// * `action` - The action type to search for
    /// * `phone_hash` - Optional phone hash to filter by
    /// * `ip_address` - Optional IP address to filter by
    /// * `since` - Only return logs created after this time
    ///
    /// # Returns
    /// * Count of failed attempts matching the criteria
    async fn count_failed_attempts(
        &self,
        action: &str,
        phone_hash: Option<&str>,
        ip_address: Option<&str>,
        since: DateTime<Utc>,
    ) -> Result<usize, DomainError>;

    /// Find suspicious activity patterns
    ///
    /// # Arguments
    /// * `ip_address` - Optional IP address to check
    /// * `since` - Time window to check
    ///
    /// # Returns
    /// * List of audit logs that may indicate suspicious activity
    async fn find_suspicious_activity(
        &self,
        ip_address: Option<&str>,
        since: DateTime<Utc>,
    ) -> Result<Vec<AuditLog>, DomainError>;
}