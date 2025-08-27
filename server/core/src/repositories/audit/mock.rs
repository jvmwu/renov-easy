//! Mock implementation of AuditLogRepository for testing.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::domain::entities::audit::{AuditLog, AuditEventType};
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
                    && (ip_address.is_none() || log.ip_address == ip_address.unwrap_or_default())
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
                    && (ip_address.is_none() || log.ip_address == ip_address.unwrap_or_default())
            })
            .cloned()
            .collect();
        
        Ok(result)
    }
    
    async fn archive_old_logs(&self) -> Result<usize, DomainError> {
        if *self.should_fail.lock().unwrap() {
            return Err(DomainError::Internal {
                message: "Mock repository error".to_string(),
            });
        }
        
        let mut logs = self.logs.lock().unwrap();
        let cutoff = Utc::now() - chrono::Duration::days(90);
        let mut archived_count = 0;
        
        for log in logs.iter_mut() {
            if log.created_at < cutoff && !log.archived {
                log.archived = true;
                log.archived_at = Some(Utc::now());
                archived_count += 1;
            }
        }
        
        Ok(archived_count)
    }
    
    async fn delete_archived_logs(&self) -> Result<usize, DomainError> {
        if *self.should_fail.lock().unwrap() {
            return Err(DomainError::Internal {
                message: "Mock repository error".to_string(),
            });
        }
        
        let mut logs = self.logs.lock().unwrap();
        let initial_count = logs.len();
        logs.retain(|log| !log.archived);
        let deleted_count = initial_count - logs.len();
        
        Ok(deleted_count)
    }
    
    async fn find_by_event_types(
        &self,
        event_types: Vec<AuditEventType>,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        limit: Option<usize>,
    ) -> Result<Vec<AuditLog>, DomainError> {
        if *self.should_fail.lock().unwrap() {
            return Err(DomainError::Internal {
                message: "Mock repository error".to_string(),
            });
        }
        
        let logs = self.logs.lock().unwrap();
        let mut result: Vec<AuditLog> = logs
            .iter()
            .filter(|log| {
                log.created_at >= from
                    && log.created_at <= to
                    && event_types.contains(&log.event_type)
            })
            .cloned()
            .collect();
        
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        if let Some(limit) = limit {
            result.truncate(limit);
        }
        
        Ok(result)
    }
}