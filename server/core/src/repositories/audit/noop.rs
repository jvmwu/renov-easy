//! No-op implementation of AuditLogRepository for when audit logging is not needed

use async_trait::async_trait;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::domain::entities::audit::{AuditLog, AuditEventType};
use crate::errors::DomainError;
use super::AuditLogRepository;

/// No-op implementation of AuditLogRepository
/// 
/// This implementation does nothing and is used when audit logging is not required.
pub struct NoOpAuditLogRepository;

impl NoOpAuditLogRepository {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AuditLogRepository for NoOpAuditLogRepository {
    async fn create(&self, _audit_log: &AuditLog) -> Result<(), DomainError> {
        // No-op implementation - just return success
        Ok(())
    }

    async fn find_by_user(
        &self,
        _user_id: Uuid,
        _limit: usize,
    ) -> Result<Vec<AuditLog>, DomainError> {
        // Return empty list
        Ok(Vec::new())
    }

    async fn find_by_phone_hash(
        &self,
        _phone_hash: &str,
        _limit: usize,
    ) -> Result<Vec<AuditLog>, DomainError> {
        // Return empty list
        Ok(Vec::new())
    }

    async fn count_failed_attempts(
        &self,
        _action: &str,
        _phone_hash: Option<&str>,
        _ip_address: Option<&str>,
        _since: DateTime<Utc>,
    ) -> Result<usize, DomainError> {
        // Return 0 failed attempts
        Ok(0)
    }

    async fn find_suspicious_activity(
        &self,
        _ip_address: Option<&str>,
        _since: DateTime<Utc>,
    ) -> Result<Vec<AuditLog>, DomainError> {
        // Return empty list
        Ok(Vec::new())
    }
    
    async fn archive_old_logs(&self) -> Result<usize, DomainError> {
        // No-op - return 0 archived
        Ok(0)
    }
    
    async fn delete_archived_logs(&self) -> Result<usize, DomainError> {
        // No-op - return 0 deleted
        Ok(0)
    }
    
    async fn find_by_event_types(
        &self,
        _event_types: Vec<AuditEventType>,
        _from: DateTime<Utc>,
        _to: DateTime<Utc>,
        _limit: Option<usize>,
    ) -> Result<Vec<AuditLog>, DomainError> {
        // No-op - return empty list
        Ok(Vec::new())
    }
}

// Also implement for () to allow simple type defaults
#[async_trait]
impl AuditLogRepository for () {
    async fn create(&self, _audit_log: &AuditLog) -> Result<(), DomainError> {
        Ok(())
    }

    async fn find_by_user(
        &self,
        _user_id: Uuid,
        _limit: usize,
    ) -> Result<Vec<AuditLog>, DomainError> {
        Ok(Vec::new())
    }

    async fn find_by_phone_hash(
        &self,
        _phone_hash: &str,
        _limit: usize,
    ) -> Result<Vec<AuditLog>, DomainError> {
        Ok(Vec::new())
    }

    async fn count_failed_attempts(
        &self,
        _action: &str,
        _phone_hash: Option<&str>,
        _ip_address: Option<&str>,
        _since: DateTime<Utc>,
    ) -> Result<usize, DomainError> {
        Ok(0)
    }

    async fn find_suspicious_activity(
        &self,
        _ip_address: Option<&str>,
        _since: DateTime<Utc>,
    ) -> Result<Vec<AuditLog>, DomainError> {
        Ok(Vec::new())
    }
    
    async fn archive_old_logs(&self) -> Result<usize, DomainError> {
        Ok(0)
    }
    
    async fn delete_archived_logs(&self) -> Result<usize, DomainError> {
        Ok(0)
    }
    
    async fn find_by_event_types(
        &self,
        _event_types: Vec<AuditEventType>,
        _from: DateTime<Utc>,
        _to: DateTime<Utc>,
        _limit: Option<usize>,
    ) -> Result<Vec<AuditLog>, DomainError> {
        Ok(Vec::new())
    }
}