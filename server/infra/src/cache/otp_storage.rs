//! OTP Redis Storage with encryption and fallback support
//!
//! This module implements secure OTP storage in Redis with:
//! - AES-256-GCM encryption for OTP codes
//! - Automatic TTL management (5 minutes)
//! - Metadata tracking (attempts, creation time, expiry)
//! - Database fallback when Redis fails
//! - Comprehensive security logging

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use re_core::errors::{DomainError, DomainResult};
use re_core::services::encryption::{
    encrypted_cache_trait::{EncryptedCacheServiceTrait, StorageBackend},
    otp_encryption::{AesGcmOtpEncryption, EncryptedOtp, OtpEncryptionConfig},
};

use crate::cache::RedisClient;
use crate::database::repositories::otp_repository::OtpRepository;

/// Default OTP expiration time in seconds (5 minutes)
const OTP_EXPIRY_SECONDS: u64 = 300;

/// Redis key prefix for OTP storage
const OTP_KEY_PREFIX: &str = "otp:encrypted";

/// Redis key prefix for OTP metadata
const OTP_METADATA_PREFIX: &str = "otp:metadata";

/// OTP metadata for Redis storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtpMetadata {
    /// Phone number (for reference)
    pub phone: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Expiration timestamp
    pub expires_at: DateTime<Utc>,
    /// Number of verification attempts
    pub attempts: u32,
    /// Maximum allowed attempts
    pub max_attempts: u32,
    /// Unique session identifier
    pub session_id: String,
    /// Whether the OTP has been used
    pub is_used: bool,
    /// Storage backend used
    pub storage_backend: StorageBackend,
}

impl From<&EncryptedOtp> for OtpMetadata {
    fn from(encrypted: &EncryptedOtp) -> Self {
        Self {
            phone: encrypted.phone.clone(),
            created_at: encrypted.created_at,
            expires_at: encrypted.expires_at,
            attempts: encrypted.attempt_count,
            max_attempts: 3,
            session_id: format!("otp_{}", encrypted.created_at.timestamp()),
            is_used: false,
            storage_backend: StorageBackend::Redis,
        }
    }
}

/// Configuration for OTP Redis storage
#[derive(Debug, Clone)]
pub struct OtpStorageConfig {
    /// OTP expiration time in seconds
    pub expiry_seconds: u64,
    /// Enable database fallback when Redis fails
    pub enable_db_fallback: bool,
    /// Maximum retry attempts for Redis operations
    pub max_redis_retries: u32,
    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
}

impl Default for OtpStorageConfig {
    fn default() -> Self {
        Self {
            expiry_seconds: OTP_EXPIRY_SECONDS,
            enable_db_fallback: true,
            max_redis_retries: 3,
            retry_delay_ms: 100,
        }
    }
}

/// OTP Redis storage implementation with encryption and fallback
pub struct OtpRedisStorage {
    /// Redis client for cache operations
    redis_client: RedisClient,
    /// OTP encryption service
    encryption_service: Arc<AesGcmOtpEncryption>,
    /// Database repository for fallback
    otp_repository: Option<Arc<OtpRepository>>,
    /// Storage configuration
    config: OtpStorageConfig,
    /// Current storage backend
    current_backend: Arc<tokio::sync::RwLock<StorageBackend>>,
}

impl OtpRedisStorage {
    /// Create a new OTP Redis storage service
    pub fn new(
        redis_client: RedisClient,
        encryption_config: OtpEncryptionConfig,
        otp_repository: Option<Arc<OtpRepository>>,
        config: OtpStorageConfig,
    ) -> DomainResult<Self> {
        let encryption_service = Arc::new(AesGcmOtpEncryption::new(encryption_config)?);

        Ok(Self {
            redis_client,
            encryption_service,
            otp_repository,
            config,
            current_backend: Arc::new(tokio::sync::RwLock::new(StorageBackend::Redis)),
        })
    }

    /// Format Redis key for encrypted OTP storage
    fn format_otp_key(phone: &str) -> String {
        format!("{}:{}", OTP_KEY_PREFIX, phone)
    }

    /// Format Redis key for OTP metadata
    fn format_metadata_key(phone: &str) -> String {
        format!("{}:{}", OTP_METADATA_PREFIX, phone)
    }

    /// Mask phone number for logging (security requirement)
    fn mask_phone(phone: &str) -> String {
        if phone.len() <= 4 {
            "****".to_string()
        } else {
            format!("***{}", &phone[phone.len() - 4..])
        }
    }

    /// Store OTP in Redis with automatic retry
    async fn store_in_redis(&self, encrypted_otp: &EncryptedOtp) -> Result<(), DomainError> {
        let otp_key = Self::format_otp_key(&encrypted_otp.phone);
        let metadata_key = Self::format_metadata_key(&encrypted_otp.phone);

        // Serialize encrypted OTP
        let otp_json = serde_json::to_string(encrypted_otp)
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to serialize encrypted OTP: {}", e)
            })?;

        // Create metadata
        let metadata = OtpMetadata::from(encrypted_otp);
        let metadata_json = serde_json::to_string(&metadata)
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to serialize OTP metadata: {}", e)
            })?;

        // Store both OTP and metadata with TTL
        for attempt in 0..self.config.max_redis_retries {
            match self.redis_client
                .set_with_expiry(&otp_key, &otp_json, self.config.expiry_seconds)
                .await
            {
                Ok(_) => {
                    // Store metadata
                    let _ = self.redis_client
                        .set_with_expiry(&metadata_key, &metadata_json, self.config.expiry_seconds)
                        .await;

                    debug!(
                        phone = Self::mask_phone(&encrypted_otp.phone),
                        key_id = &encrypted_otp.key_id,
                        expires_at = %encrypted_otp.expires_at,
                        attempt = attempt + 1,
                        "Successfully stored encrypted OTP in Redis"
                    );

                    return Ok(());
                }
                Err(e) if attempt < self.config.max_redis_retries - 1 => {
                    warn!(
                        phone = Self::mask_phone(&encrypted_otp.phone),
                        error = %e,
                        attempt = attempt + 1,
                        max_attempts = self.config.max_redis_retries,
                        "Redis storage failed, retrying..."
                    );
                    tokio::time::sleep(tokio::time::Duration::from_millis(self.config.retry_delay_ms)).await;
                }
                Err(e) => return Err(DomainError::Internal {
                    message: format!("Failed to store OTP in Redis: {}", e),
                }),
            }
        }

        Err(DomainError::Internal {
            message: "Failed to store OTP in Redis after all retries".to_string()
        })
    }

    /// Store OTP in database (fallback)
    async fn store_in_database(&self, encrypted_otp: &EncryptedOtp) -> DomainResult<()> {
        if let Some(repo) = &self.otp_repository {
            warn!(
                phone = Self::mask_phone(&encrypted_otp.phone),
                "Falling back to database storage for OTP"
            );

            repo.store_encrypted_otp(encrypted_otp).await?;

            info!(
                phone = Self::mask_phone(&encrypted_otp.phone),
                "Successfully stored encrypted OTP in database (fallback)"
            );

            Ok(())
        } else {
            Err(DomainError::Internal {
                message: "Database fallback not configured and Redis is unavailable".to_string(),
            })
        }
    }

    /// Retrieve OTP from Redis
    async fn get_from_redis(&self, phone: &str) -> Result<Option<EncryptedOtp>, DomainError> {
        let otp_key = Self::format_otp_key(phone);

        match self.redis_client.get(&otp_key).await
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to get OTP from Redis: {}", e),
            })? {
            Some(otp_json) => {
                let encrypted_otp: EncryptedOtp = serde_json::from_str(&otp_json)
                    .map_err(|e| DomainError::Internal {
                        message: format!("Failed to deserialize encrypted OTP: {}", e)
                    })?;

                debug!(
                    phone = Self::mask_phone(phone),
                    "Retrieved encrypted OTP from Redis"
                );

                Ok(Some(encrypted_otp))
            }
            None => Ok(None),
        }
    }

    /// Retrieve OTP from database (fallback)
    async fn get_from_database(&self, phone: &str) -> DomainResult<Option<EncryptedOtp>> {
        if let Some(repo) = &self.otp_repository {
            debug!(
                phone = Self::mask_phone(phone),
                "Retrieving OTP from database (fallback)"
            );

            repo.get_encrypted_otp(phone).await
        } else {
            Ok(None)
        }
    }

    /// Invalidate previous OTP codes for a phone number
    async fn invalidate_previous_codes(&self, phone: &str) -> DomainResult<()> {
        let otp_key = Self::format_otp_key(phone);
        let metadata_key = Self::format_metadata_key(phone);

        // Delete from Redis
        let _ = self.redis_client.delete(&otp_key).await;
        let _ = self.redis_client.delete(&metadata_key).await;

        // Delete from database if configured
        if let Some(repo) = &self.otp_repository {
            let _ = repo.delete_otp(phone).await;
        }

        info!(
            phone = Self::mask_phone(phone),
            "Invalidated previous OTP codes"
        );

        Ok(())
    }
}

#[async_trait]
impl EncryptedCacheServiceTrait for OtpRedisStorage {
    async fn store_encrypted_otp(
        &self,
        encrypted_otp: &EncryptedOtp,
    ) -> DomainResult<StorageBackend> {
        // Invalidate any existing codes for this phone
        self.invalidate_previous_codes(&encrypted_otp.phone).await?;

        // Try Redis first
        match self.store_in_redis(encrypted_otp).await {
            Ok(_) => {
                *self.current_backend.write().await = StorageBackend::Redis;

                info!(
                    phone = Self::mask_phone(&encrypted_otp.phone),
                    backend = "Redis",
                    event = "otp_stored",
                    session_id = encrypted_otp.created_at.timestamp(),
                    "Encrypted OTP stored successfully"
                );

                Ok(StorageBackend::Redis)
            }
            Err(redis_error) => {
                error!(
                    phone = Self::mask_phone(&encrypted_otp.phone),
                    error = %redis_error,
                    event = "redis_storage_failed",
                    "Failed to store OTP in Redis, attempting fallback"
                );

                // Fallback to database if enabled
                if self.config.enable_db_fallback {
                    self.store_in_database(encrypted_otp).await?;
                    *self.current_backend.write().await = StorageBackend::Database;

                    warn!(
                        phone = Self::mask_phone(&encrypted_otp.phone),
                        backend = "Database",
                        event = "otp_stored_fallback",
                        "Encrypted OTP stored in database (fallback)"
                    );

                    Ok(StorageBackend::Database)
                } else {
                    Err(DomainError::Internal {
                        message: format!("Redis storage failed and fallback is disabled: {}", redis_error),
                    })
                }
            }
        }
    }

    async fn get_encrypted_otp(&self, phone: &str) -> DomainResult<Option<EncryptedOtp>> {
        // Try Redis first
        match self.get_from_redis(phone).await {
            Ok(Some(otp)) => {
                debug!(
                    phone = Self::mask_phone(phone),
                    backend = "Redis",
                    "Retrieved OTP from Redis"
                );
                Ok(Some(otp))
            }
            Ok(None) => {
                // Not found in Redis, try database if fallback is enabled
                if self.config.enable_db_fallback {
                    self.get_from_database(phone).await
                } else {
                    Ok(None)
                }
            }
            Err(redis_error) => {
                warn!(
                    phone = Self::mask_phone(phone),
                    error = %redis_error,
                    "Failed to get OTP from Redis, trying fallback"
                );

                // Fallback to database if enabled
                if self.config.enable_db_fallback {
                    self.get_from_database(phone).await
                } else {
                    Err(DomainError::Internal {
                        message: format!("Redis retrieval failed: {}", redis_error),
                    })
                }
            }
        }
    }

    async fn increment_attempt_count(&self, phone: &str) -> DomainResult<u32> {
        let metadata_key = Self::format_metadata_key(phone);

        // Get current metadata
        match self.redis_client.get(&metadata_key).await {
            Ok(Some(metadata_json)) => {
                let mut metadata: OtpMetadata = serde_json::from_str(&metadata_json)
                    .map_err(|e| DomainError::Internal {
                        message: format!("Failed to deserialize metadata: {}", e)
                    })?;

                // Increment attempts
                metadata.attempts += 1;

                // Update in Redis
                let updated_json = serde_json::to_string(&metadata)
                    .map_err(|e| DomainError::Internal {
                        message: format!("Failed to serialize metadata: {}", e)
                    })?;

                // Calculate remaining TTL
                let ttl = self.redis_client.ttl(&metadata_key).await
                    .unwrap_or(Some(self.config.expiry_seconds as i64))
                    .unwrap_or(self.config.expiry_seconds as i64) as u64;

                self.redis_client
                    .set_with_expiry(&metadata_key, &updated_json, ttl)
                    .await
                    .map_err(|e| DomainError::Internal {
                        message: format!("Failed to update metadata: {}", e)
                    })?;

                debug!(
                    phone = Self::mask_phone(phone),
                    attempts = metadata.attempts,
                    "Incremented OTP attempt count"
                );

                Ok(metadata.attempts)
            }
            _ => {
                // If metadata doesn't exist, check if we have the OTP in database
                if self.config.enable_db_fallback {
                    if let Some(repo) = &self.otp_repository {
                        repo.increment_attempt_count(phone).await
                    } else {
                        Ok(1)
                    }
                } else {
                    Ok(1)
                }
            }
        }
    }

    async fn encrypted_otp_exists(&self, phone: &str) -> DomainResult<bool> {
        let otp_key = Self::format_otp_key(phone);

        // Check Redis first
        match self.redis_client.exists(&otp_key).await {
            Ok(true) => Ok(true),
            Ok(false) => {
                // Not in Redis, check database if fallback is enabled
                if self.config.enable_db_fallback {
                    if let Some(repo) = &self.otp_repository {
                        repo.otp_exists(phone).await
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }
            Err(e) => {
                warn!(
                    phone = Self::mask_phone(phone),
                    error = %e,
                    "Failed to check OTP existence in Redis"
                );

                // Fallback to database if enabled
                if self.config.enable_db_fallback {
                    if let Some(repo) = &self.otp_repository {
                        repo.otp_exists(phone).await
                    } else {
                        Ok(false)
                    }
                } else {
                    Err(DomainError::Internal {
                        message: format!("Redis check failed: {}", e),
                    })
                }
            }
        }
    }

    async fn get_encrypted_otp_ttl(&self, phone: &str) -> DomainResult<Option<i64>> {
        let otp_key = Self::format_otp_key(phone);

        self.redis_client.ttl(&otp_key).await
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to get TTL: {}", e)
            })
    }

    async fn clear_encrypted_otp(&self, phone: &str) -> DomainResult<()> {
        let otp_key = Self::format_otp_key(phone);
        let metadata_key = Self::format_metadata_key(phone);

        // Clear from Redis
        let _ = self.redis_client.delete(&otp_key).await;
        let _ = self.redis_client.delete(&metadata_key).await;

        // Clear from database if configured
        if let Some(repo) = &self.otp_repository {
            let _ = repo.delete_otp(phone).await;
        }

        info!(
            phone = Self::mask_phone(phone),
            event = "otp_cleared",
            "Cleared encrypted OTP data"
        );

        Ok(())
    }

    async fn get_current_backend(&self) -> StorageBackend {
        *self.current_backend.read().await
    }

    async fn is_redis_available(&self) -> bool {
        // Try a simple ping to check Redis availability
        self.redis_client.get("ping:test").await.is_ok()
    }
}
