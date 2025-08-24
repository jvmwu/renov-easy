//! OTP database repository for fallback storage
//!
//! This module provides database storage for OTPs when Redis is unavailable.
//! It implements the fallback mechanism required by requirement 8.2.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{MySql, Pool, Row};
use tracing::{debug, error, info, warn};

use re_core::errors::{DomainError, DomainResult};
use re_core::services::encryption::otp_encryption::EncryptedOtp;

/// OTP database repository for fallback storage
pub struct OtpRepository {
    /// Database connection pool
    pool: Pool<MySql>,
}

impl OtpRepository {
    /// Create a new OTP repository
    pub fn new(pool: Pool<MySql>) -> Self {
        Self { pool }
    }

    /// Store an encrypted OTP in the database
    pub async fn store_encrypted_otp(&self, encrypted_otp: &EncryptedOtp) -> DomainResult<()> {
        // First, delete any existing OTP for this phone
        self.delete_otp(&encrypted_otp.phone).await?;

        let query = r#"
            INSERT INTO otp_fallback (
                phone, ciphertext, nonce, key_id,
                created_at, expires_at, attempt_count
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(&encrypted_otp.phone)
            .bind(&encrypted_otp.ciphertext)
            .bind(&encrypted_otp.nonce)
            .bind(&encrypted_otp.key_id)
            .bind(encrypted_otp.created_at)
            .bind(encrypted_otp.expires_at)
            .bind(encrypted_otp.attempt_count)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    phone = Self::mask_phone(&encrypted_otp.phone),
                    error = %e,
                    "Failed to store OTP in database"
                );
                DomainError::Internal {
                    message: format!("Failed to store OTP in database: {}", e),
                }
            })?;

        info!(
            phone = Self::mask_phone(&encrypted_otp.phone),
            "Stored encrypted OTP in database (fallback)"
        );

        Ok(())
    }

    /// Retrieve an encrypted OTP from the database
    pub async fn get_encrypted_otp(&self, phone: &str) -> DomainResult<Option<EncryptedOtp>> {
        let query = r#"
            SELECT ciphertext, nonce, key_id, created_at,
                   expires_at, attempt_count
            FROM otp_fallback
            WHERE phone = ? AND expires_at > NOW()
            ORDER BY created_at DESC
            LIMIT 1
        "#;

        let result = sqlx::query(query)
            .bind(phone)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    phone = Self::mask_phone(phone),
                    error = %e,
                    "Failed to retrieve OTP from database"
                );
                DomainError::Internal {
                    message: format!("Failed to retrieve OTP from database: {}", e),
                }
            })?;

        match result {
            Some(row) => {
                let encrypted_otp = EncryptedOtp {
                    ciphertext: row.try_get("ciphertext").map_err(|e| DomainError::Internal {
                        message: format!("Failed to get ciphertext: {}", e),
                    })?,
                    nonce: row.try_get("nonce").map_err(|e| DomainError::Internal {
                        message: format!("Failed to get nonce: {}", e),
                    })?,
                    key_id: row.try_get("key_id").map_err(|e| DomainError::Internal {
                        message: format!("Failed to get key_id: {}", e),
                    })?,
                    created_at: row.try_get("created_at").map_err(|e| DomainError::Internal {
                        message: format!("Failed to get created_at: {}", e),
                    })?,
                    expires_at: row.try_get("expires_at").map_err(|e| DomainError::Internal {
                        message: format!("Failed to get expires_at: {}", e),
                    })?,
                    attempt_count: row.try_get("attempt_count").map_err(|e| DomainError::Internal {
                        message: format!("Failed to get attempt_count: {}", e),
                    })?,
                    phone: phone.to_string(),
                };

                debug!(
                    phone = Self::mask_phone(phone),
                    "Retrieved encrypted OTP from database"
                );

                Ok(Some(encrypted_otp))
            }
            None => {
                debug!(
                    phone = Self::mask_phone(phone),
                    "No valid OTP found in database"
                );
                Ok(None)
            }
        }
    }

    /// Increment the attempt count for an OTP
    pub async fn increment_attempt_count(&self, phone: &str) -> DomainResult<u32> {
        let query = r#"
            UPDATE otp_fallback
            SET attempt_count = attempt_count + 1
            WHERE phone = ? AND expires_at > NOW()
            ORDER BY created_at DESC
            LIMIT 1
        "#;

        sqlx::query(query)
            .bind(phone)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    phone = Self::mask_phone(phone),
                    error = %e,
                    "Failed to increment attempt count"
                );
                DomainError::Internal {
                    message: format!("Failed to increment attempt count: {}", e),
                }
            })?;

        // Get the updated count
        let count_query = r#"
            SELECT attempt_count
            FROM otp_fallback
            WHERE phone = ? AND expires_at > NOW()
            ORDER BY created_at DESC
            LIMIT 1
        "#;

        let count = sqlx::query(count_query)
            .bind(phone)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to get attempt count: {}", e),
            })?
            .and_then(|row| row.try_get::<u32, _>("attempt_count").ok())
            .unwrap_or(1);

        debug!(
            phone = Self::mask_phone(phone),
            attempt_count = count,
            "Incremented OTP attempt count"
        );

        Ok(count)
    }

    /// Check if an OTP exists for a phone number
    pub async fn otp_exists(&self, phone: &str) -> DomainResult<bool> {
        let query = r#"
            SELECT 1
            FROM otp_fallback
            WHERE phone = ? AND expires_at > NOW()
            LIMIT 1
        "#;

        let exists = sqlx::query(query)
            .bind(phone)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to check OTP existence: {}", e),
            })?
            .is_some();

        Ok(exists)
    }

    /// Delete an OTP for a phone number
    pub async fn delete_otp(&self, phone: &str) -> DomainResult<()> {
        let query = "DELETE FROM otp_fallback WHERE phone = ?";

        sqlx::query(query)
            .bind(phone)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    phone = Self::mask_phone(phone),
                    error = %e,
                    "Failed to delete OTP from database"
                );
                DomainError::Internal {
                    message: format!("Failed to delete OTP: {}", e),
                }
            })?;

        debug!(
            phone = Self::mask_phone(phone),
            "Deleted OTP from database"
        );

        Ok(())
    }

    /// Clean up expired OTPs (maintenance task)
    pub async fn cleanup_expired_otps(&self) -> DomainResult<u64> {
        let query = "DELETE FROM otp_fallback WHERE expires_at <= NOW()";

        let result = sqlx::query(query)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to cleanup expired OTPs: {}", e),
            })?;

        let deleted_count = result.rows_affected();

        if deleted_count > 0 {
            info!(
                deleted_count = deleted_count,
                "Cleaned up expired OTPs from database"
            );
        }

        Ok(deleted_count)
    }

    /// Mask phone number for logging (security requirement)
    fn mask_phone(phone: &str) -> String {
        if phone.len() <= 4 {
            "****".to_string()
        } else {
            format!("***{}", &phone[phone.len() - 4..])
        }
    }
}
