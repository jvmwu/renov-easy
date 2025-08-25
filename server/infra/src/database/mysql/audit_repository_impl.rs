//! MySQL implementation of the AuditLogRepository trait.
//!
//! This module provides the concrete implementation of audit log persistence
//! using MySQL database with SQLx. It handles all database operations for
//! security event logging and compliance tracking.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::{MySqlPool, Row};
use uuid::Uuid;

use re_core::domain::entities::audit::{AuditEventType, AuditLog};
use re_core::errors::DomainError;
use re_core::repositories::audit::AuditLogRepository;

/// MySQL implementation of AuditLogRepository
///
/// This implementation uses SQLx for database operations and stores
/// audit logs in the auth_audit_log table for immutable security tracking.
pub struct MySqlAuditLogRepository {
    /// Database connection pool
    pool: MySqlPool,
}

impl MySqlAuditLogRepository {
    /// Create a new MySQL audit log repository
    ///
    /// # Arguments
    /// * `pool` - MySQL connection pool from SQLx
    ///
    /// # Returns
    /// A new instance of MySqlAuditLogRepository
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Convert database row to AuditLog entity
    ///
    /// Maps database columns to AuditLog struct fields
    fn row_to_audit_log(row: &sqlx::mysql::MySqlRow) -> Result<AuditLog, DomainError> {
        let id: String = row
            .try_get("id")
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to get id: {}", e),
            })?;

        let event_type_str: String = row
            .try_get("event_type")
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to get event_type: {}", e),
            })?;

        let event_type = AuditEventType::from_str(&event_type_str)
            .ok_or_else(|| DomainError::Internal {
                message: format!("Unknown event type: {}", event_type_str),
            })?;

        let user_id: Option<String> = row
            .try_get("user_id")
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to get user_id: {}", e),
            })?;

        let user_id = user_id
            .map(|id| Uuid::parse_str(&id))
            .transpose()
            .map_err(|e| DomainError::Internal {
                message: format!("Invalid user UUID: {}", e),
            })?;

        let token_id: Option<String> = row
            .try_get("token_id")
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to get token_id: {}", e),
            })?;

        let token_id = token_id
            .map(|id| Uuid::parse_str(&id))
            .transpose()
            .map_err(|e| DomainError::Internal {
                message: format!("Invalid token UUID: {}", e),
            })?;

        let event_data: Option<serde_json::Value> = row
            .try_get("event_data")
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to get event_data: {}", e),
            })?;

        let archived_at: Option<DateTime<Utc>> = row
            .try_get("archived_at")
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to get archived_at: {}", e),
            })?;

        Ok(AuditLog {
            id: Uuid::parse_str(&id).map_err(|e| DomainError::Internal {
                message: format!("Invalid UUID: {}", e),
            })?,
            event_type,
            user_id,
            phone_masked: row.try_get("phone_masked").map_err(|e| DomainError::Internal {
                message: format!("Failed to get phone_masked: {}", e),
            })?,
            phone_hash: row.try_get("phone_hash").map_err(|e| DomainError::Internal {
                message: format!("Failed to get phone_hash: {}", e),
            })?,
            ip_address: row.try_get("ip_address").map_err(|e| DomainError::Internal {
                message: format!("Failed to get ip_address: {}", e),
            })?,
            user_agent: row.try_get("user_agent").map_err(|e| DomainError::Internal {
                message: format!("Failed to get user_agent: {}", e),
            })?,
            device_info: row.try_get("device_info").map_err(|e| DomainError::Internal {
                message: format!("Failed to get device_info: {}", e),
            })?,
            event_data,
            failure_reason: row
                .try_get("failure_reason")
                .map_err(|e| DomainError::Internal {
                    message: format!("Failed to get failure_reason: {}", e),
                })?,
            token_id,
            rate_limit_type: row
                .try_get("rate_limit_type")
                .map_err(|e| DomainError::Internal {
                    message: format!("Failed to get rate_limit_type: {}", e),
                })?,
            success: row.try_get("success").map_err(|e| DomainError::Internal {
                message: format!("Failed to get success: {}", e),
            })?,
            action: row.try_get("action").map_err(|e| DomainError::Internal {
                message: format!("Failed to get action: {}", e),
            })?,
            error_message: row
                .try_get("error_message")
                .map_err(|e| DomainError::Internal {
                    message: format!("Failed to get error_message: {}", e),
                })?,
            created_at: row
                .try_get::<DateTime<Utc>, _>("created_at")
                .map_err(|e| DomainError::Internal {
                    message: format!("Failed to get created_at: {}", e),
                })?,
            archived: row.try_get("archived").map_err(|e| DomainError::Internal {
                message: format!("Failed to get archived: {}", e),
            })?,
            archived_at,
        })
    }
}

#[async_trait]
impl AuditLogRepository for MySqlAuditLogRepository {
    async fn create(&self, audit_log: &AuditLog) -> Result<(), DomainError> {
        let query = r#"
            INSERT INTO auth_audit_log (
                id, event_type, user_id, phone_masked, phone_hash,
                ip_address, user_agent, device_info, action, success,
                error_message, failure_reason, token_id, rate_limit_type,
                event_data, created_at, archived, archived_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        // Convert event_data to JSON string if present
        let event_data_json = audit_log
            .event_data
            .as_ref()
            .map(|data| serde_json::to_string(data))
            .transpose()
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to serialize event_data: {}", e),
            })?;

        sqlx::query(query)
            .bind(audit_log.id.to_string())
            .bind(audit_log.event_type.as_str())
            .bind(audit_log.user_id.map(|id| id.to_string()))
            .bind(&audit_log.phone_masked)
            .bind(&audit_log.phone_hash)
            .bind(&audit_log.ip_address)
            .bind(&audit_log.user_agent)
            .bind(&audit_log.device_info)
            .bind(&audit_log.action)
            .bind(audit_log.success)
            .bind(&audit_log.error_message)
            .bind(&audit_log.failure_reason)
            .bind(audit_log.token_id.map(|id| id.to_string()))
            .bind(&audit_log.rate_limit_type)
            .bind(event_data_json)
            .bind(audit_log.created_at)
            .bind(audit_log.archived)
            .bind(audit_log.archived_at)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to create audit log: {}", e),
            })?;

        Ok(())
    }

    async fn find_by_user(
        &self,
        user_id: Uuid,
        limit: usize,
    ) -> Result<Vec<AuditLog>, DomainError> {
        let query = r#"
            SELECT id, event_type, user_id, phone_masked, phone_hash,
                   ip_address, user_agent, device_info, action, success,
                   error_message, failure_reason, token_id, rate_limit_type,
                   event_data, created_at, archived, archived_at
            FROM auth_audit_log
            WHERE user_id = ?
            ORDER BY created_at DESC
            LIMIT ?
        "#;

        let rows = sqlx::query(query)
            .bind(user_id.to_string())
            .bind(limit as i32)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to find audit logs by user: {}", e),
            })?;

        rows.iter()
            .map(Self::row_to_audit_log)
            .collect::<Result<Vec<_>, _>>()
    }

    async fn find_by_phone_hash(
        &self,
        phone_hash: &str,
        limit: usize,
    ) -> Result<Vec<AuditLog>, DomainError> {
        let query = r#"
            SELECT id, event_type, user_id, phone_masked, phone_hash,
                   ip_address, user_agent, device_info, action, success,
                   error_message, failure_reason, token_id, rate_limit_type,
                   event_data, created_at, archived, archived_at
            FROM auth_audit_log
            WHERE phone_hash = ?
            ORDER BY created_at DESC
            LIMIT ?
        "#;

        let rows = sqlx::query(query)
            .bind(phone_hash)
            .bind(limit as i32)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to find audit logs by phone hash: {}", e),
            })?;

        rows.iter()
            .map(Self::row_to_audit_log)
            .collect::<Result<Vec<_>, _>>()
    }

    async fn count_failed_attempts(
        &self,
        action: &str,
        phone_hash: Option<&str>,
        ip_address: Option<&str>,
        since: DateTime<Utc>,
    ) -> Result<usize, DomainError> {
        let mut query = String::from(
            r#"
            SELECT COUNT(*) as count
            FROM auth_audit_log
            WHERE action = ?
            AND success = FALSE
            AND created_at >= ?
            "#,
        );

        let mut bindings = vec![];
        bindings.push(action.to_string());
        bindings.push(since.to_rfc3339());

        if let Some(phone) = phone_hash {
            query.push_str(" AND phone_hash = ?");
            bindings.push(phone.to_string());
        }

        if let Some(ip) = ip_address {
            query.push_str(" AND ip_address = ?");
            bindings.push(ip.to_string());
        }

        let mut query_builder = sqlx::query(&query);
        for binding in bindings {
            query_builder = query_builder.bind(binding);
        }

        let row = query_builder
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to count failed attempts: {}", e),
            })?;

        let count: i64 = row.try_get("count").map_err(|e| DomainError::Internal {
            message: format!("Failed to get count: {}", e),
        })?;

        Ok(count as usize)
    }

    async fn find_suspicious_activity(
        &self,
        ip_address: Option<&str>,
        since: DateTime<Utc>,
    ) -> Result<Vec<AuditLog>, DomainError> {
        let query = if let Some(ip) = ip_address {
            r#"
                SELECT id, event_type, user_id, phone_masked, phone_hash,
                       ip_address, user_agent, device_info, action, success,
                       error_message, failure_reason, token_id, rate_limit_type,
                       event_data, created_at, archived, archived_at
                FROM auth_audit_log
                WHERE created_at >= ?
                AND ip_address = ?
                AND (
                    success = FALSE
                    OR event_type IN ('RATE_LIMIT_EXCEEDED', 'SUSPICIOUS_ACTIVITY', 'INVALID_TOKEN_USAGE')
                )
                ORDER BY created_at DESC
            "#
        } else {
            r#"
                SELECT id, event_type, user_id, phone_masked, phone_hash,
                       ip_address, user_agent, device_info, action, success,
                       error_message, failure_reason, token_id, rate_limit_type,
                       event_data, created_at, archived, archived_at
                FROM auth_audit_log
                WHERE created_at >= ?
                AND (
                    success = FALSE
                    OR event_type IN ('RATE_LIMIT_EXCEEDED', 'SUSPICIOUS_ACTIVITY', 'INVALID_TOKEN_USAGE')
                )
                ORDER BY created_at DESC
            "#
        };

        let rows = if let Some(ip) = ip_address {
            sqlx::query(query)
                .bind(since)
                .bind(ip)
                .fetch_all(&self.pool)
                .await
        } else {
            sqlx::query(query)
                .bind(since)
                .fetch_all(&self.pool)
                .await
        }
        .map_err(|e| DomainError::Internal {
            message: format!("Failed to find suspicious activity: {}", e),
        })?;

        rows.iter()
            .map(Self::row_to_audit_log)
            .collect::<Result<Vec<_>, _>>()
    }

    async fn archive_old_logs(&self) -> Result<usize, DomainError> {
        let query = r#"
            UPDATE auth_audit_log
            SET archived = TRUE,
                archived_at = NOW()
            WHERE created_at < DATE_SUB(NOW(), INTERVAL 90 DAY)
            AND archived = FALSE
        "#;

        let result = sqlx::query(query)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to archive old logs: {}", e),
            })?;

        Ok(result.rows_affected() as usize)
    }

    async fn delete_archived_logs(&self) -> Result<usize, DomainError> {
        let query = r#"
            DELETE FROM auth_audit_log
            WHERE archived = TRUE
            AND archived_at < DATE_SUB(NOW(), INTERVAL 7 DAY)
        "#;

        let result = sqlx::query(query)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to delete archived logs: {}", e),
            })?;

        Ok(result.rows_affected() as usize)
    }
}
