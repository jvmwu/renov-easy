//! Mock implementation of AuditLogRepository for testing.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::domain::entities::audit::AuditLog;
use crate::errors::DomainError;

use super::AuditLogRepository;

/// Mock implementation of AuditLogRepository for testing
pub struct MockAuditLogRepository {
    logs: Arc<Mutex<Vec<AuditLog>>>,
    should_fail: Arc<Mutex<bool>>,
}

impl MockAuditLogRepository {
    /// Create a new mock repository
    pub fn new() -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
            should_fail: Arc::new(Mutex::new(false)),
        }
    }

    /// Set whether operations should fail
    pub fn set_should_fail(&self, should_fail: bool) {
        *self.should_fail.lock().unwrap() = should_fail;
    }

    /// Get all stored logs for testing
    pub fn get_all_logs(&self) -> Vec<AuditLog> {
        self.logs.lock().unwrap().clone()
    }

    /// Clear all logs
    pub fn clear(&self) {
        self.logs.lock().unwrap().clear();
    }
}

impl Default for MockAuditLogRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuditLogRepository for MockAuditLogRepository {
    async fn create(&self, audit_log: &AuditLog) -> Result<(), DomainError> {
        if *self.should_fail.lock().unwrap() {
            return Err(DomainError::Internal {
                message: "Mock repository error".to_string(),
            });
        }

        let mut logs = self.logs.lock().unwrap();
        logs.push(audit_log.clone());
        Ok(())
    }

    async fn find_by_user(
        &self,
        user_id: Uuid,
        limit: usize,
    ) -> Result<Vec<AuditLog>, DomainError> {
        if *self.should_fail.lock().unwrap() {
            return Err(DomainError::Internal {
                message: "Mock repository error".to_string(),
            });
        }

        let logs = self.logs.lock().unwrap();
        let mut result: Vec<AuditLog> = logs
            .iter()
            .filter(|log| log.user_id == Some(user_id))
            .cloned()
            .collect();
        
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        result.truncate(limit);
        Ok(result)
    }

    async fn find_by_phone_hash(
        &self,
        phone_hash: &str,
        limit: usize,
    ) -> Result<Vec<AuditLog>, DomainError> {
        if *self.should_fail.lock().unwrap() {
            return Err(DomainError::Internal {
                message: "Mock repository error".to_string(),
            });
        }

        let logs = self.logs.lock().unwrap();
        let mut result: Vec<AuditLog> = logs
            .iter()
            .filter(|log| log.phone_hash.as_deref() == Some(phone_hash))
            .cloned()
            .collect();
        
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        result.truncate(limit);
        Ok(result)
    }

    async fn count_failed_attempts(
        &self,
        action: &str,
        phone_hash: Option<&str>,
        ip_address: Option<&str>,
        since: DateTime<Utc>,
    ) -> Result<usize, DomainError> {
        if *self.should_fail.lock().unwrap() {
            return Err(DomainError::Internal {
                message: "Mock repository error".to_string(),
            });
        }

        let logs = self.logs.lock().unwrap();
        let count = logs
            .iter()
            .filter(|log| {
                log.action == action
                    && !log.success
                    && log.created_at >= since
                    && (phone_hash.is_none() || log.phone_hash.as_deref() == phone_hash)
                    && (ip_address.is_none() || log.ip_address.as_deref() == ip_address)
            })
            .count();
        
        Ok(count)
    }

    async fn find_suspicious_activity(
        &self,
        ip_address: Option<&str>,
        since: DateTime<Utc>,
    ) -> Result<Vec<AuditLog>, DomainError> {
        if *self.should_fail.lock().unwrap() {
            return Err(DomainError::Internal {
                message: "Mock repository error".to_string(),
            });
        }

        let logs = self.logs.lock().unwrap();
        let result: Vec<AuditLog> = logs
            .iter()
            .filter(|log| {
                log.created_at >= since
                    && !log.success
                    && (ip_address.is_none() || log.ip_address.as_deref() == ip_address)
            })
            .cloned()
            .collect();
        
        Ok(result)
    }
}