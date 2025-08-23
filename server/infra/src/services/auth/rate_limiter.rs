//! Redis-based rate limiter implementation for authentication services

use async_trait::async_trait;
use chrono::Utc;
use redis::AsyncCommands;
use std::sync::Arc;

use re_core::{DomainError, DomainResult};
use re_core::RateLimiterTrait;
use re_shared::RateLimitConfig;

use crate::cache::redis_client::RedisClient;

/// Redis-based implementation of the rate limiter trait
pub struct RedisRateLimiter {
    redis_client: Arc<RedisClient>,
    config: RateLimitConfig,
}

impl RedisRateLimiter {
    /// Create a new Redis-based rate limiter
    pub fn new(redis_client: Arc<RedisClient>, config: RateLimitConfig) -> Self {
        Self {
            redis_client,
            config,
        }
    }

    /// Check if a phone number is locked due to failed attempts
    pub async fn is_phone_locked(&self, phone: &str) -> DomainResult<bool> {
        let key = format!("account_lock:phone:{}", hash_phone(phone));
        let mut conn = self.redis_client.get_connection();
        
        let exists: bool = conn.exists(&key).await
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to check lock status: {}", e),
            })?;
        
        Ok(exists)
    }

    /// Check if an IP is locked
    pub async fn is_ip_locked(&self, ip: &str) -> DomainResult<bool> {
        let key = format!("account_lock:ip:{}", ip);
        let mut conn = self.redis_client.get_connection();
        
        let exists: bool = conn.exists(&key).await
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to check lock status: {}", e),
            })?;
        
        Ok(exists)
    }

    /// Lock a phone number for the configured duration
    async fn lock_phone(&self, phone: &str) -> DomainResult<()> {
        let key = format!("account_lock:phone:{}", hash_phone(phone));
        let mut conn = self.redis_client.get_connection();
        
        let lockout_duration = self.config.auth.account_lock_duration;
        conn.set_ex(&key, "locked", lockout_duration as u64)
            .await
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to lock phone: {}", e),
            })?;
        
        // TODO: Add audit logging when AuditRepository is available
        
        Ok(())
    }

    /// Lock an IP address for the configured duration
    async fn lock_ip(&self, ip: &str) -> DomainResult<()> {
        let key = format!("account_lock:ip:{}", ip);
        let mut conn = self.redis_client.get_connection();
        
        let lockout_duration = self.config.auth.account_lock_duration;
        conn.set_ex(&key, "locked", lockout_duration as u64)
            .await
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to lock IP: {}", e),
            })?;
        
        // TODO: Add audit logging when AuditRepository is available
        
        Ok(())
    }

    /// Check rate limit for a specific key using sliding window algorithm
    async fn check_rate_limit(
        &self,
        key: &str,
        limit: u32,
        window_seconds: u64,
    ) -> DomainResult<RateLimitStatus> {
        let mut conn = self.redis_client.get_connection();
        
        let now = Utc::now().timestamp_millis();
        let window_start = now - (window_seconds as i64 * 1000);
        
        // Remove old entries outside the window
        // Remove expired entries using raw Redis command
        let _: redis::RedisResult<i64> = redis::cmd("ZREMRANGEBYSCORE")
            .arg(key)
            .arg("-inf")
            .arg(window_start)
            .query_async(&mut conn)
            .await;
        
        // Count current entries in the window
        let count: u32 = conn.zcount(key, window_start, "+inf").await
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to count rate limit: {}", e),
            })?;
        
        if count >= limit {
            // Get the oldest entry to calculate retry time
            let oldest: Vec<(String, i64)> = conn.zrangebyscore_limit_withscores(
                key, 
                window_start, 
                "+inf", 
                0, 
                1
            ).await
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to get rate limit window: {}", e),
            })?;
            
            let retry_after = if let Some((_, timestamp)) = oldest.first() {
                ((timestamp + (window_seconds as i64 * 1000) - now) / 1000).max(1) as u64
            } else {
                window_seconds
            };
            
            Ok(RateLimitStatus::Exceeded {
                retry_after_seconds: retry_after,
                limit,
                window_seconds,
            })
        } else {
            // Add current request to the window
            conn.zadd(key, &now.to_string(), now).await
                .map_err(|e| DomainError::Internal {
                    message: format!("Failed to update rate limit: {}", e),
                })?;
            
            // Set expiry on the key
            conn.expire(key, window_seconds as i64).await
                .map_err(|e| DomainError::Internal {
                    message: format!("Failed to set expiry: {}", e),
                })?;
            
            Ok(RateLimitStatus::Ok {
                remaining: limit - count - 1,
                limit,
                window_seconds,
            })
        }
    }

    /// Check phone SMS rate limit
    pub async fn check_phone_sms_limit(&self, phone: &str) -> DomainResult<RateLimitStatus> {
        // First check if phone is locked
        if self.is_phone_locked(phone).await? {
            let ttl = self.get_lock_ttl(&format!("account_lock:phone:{}", hash_phone(phone))).await?;
            return Ok(RateLimitStatus::Locked {
                retry_after_seconds: ttl.unwrap_or(3600),
                reason: "Phone locked due to excessive failed attempts".to_string(),
            });
        }
        
        let key = format!("rate_limit:sms:{}", hash_phone(phone));
        let limit = self.config.sms.per_phone_per_hour;
        let window = 3600u64; // 1 hour window for SMS
        self.check_rate_limit(&key, limit, window).await
    }

    /// Check IP verification limit
    pub async fn check_ip_verification_limit(&self, ip: &str) -> DomainResult<RateLimitStatus> {
        // First check if IP is locked
        if self.is_ip_locked(ip).await? {
            let ttl = self.get_lock_ttl(&format!("account_lock:ip:{}", ip)).await?;
            return Ok(RateLimitStatus::Locked {
                retry_after_seconds: ttl.unwrap_or(3600),
                reason: "IP locked due to excessive requests".to_string(),
            });
        }
        
        let key = format!("rate_limit:ip_verification:{}", ip);
        let limit = self.config.auth.login_per_ip_per_hour;
        let window = 3600; // 1 hour in seconds
        self.check_rate_limit(&key, limit, window).await
    }

    /// Get the status of all rate limits for a phone number
    pub async fn get_phone_status(&self, phone: &str) -> DomainResult<RateLimitInfo> {
        let is_locked = self.is_phone_locked(phone).await?;
        let lock_ttl = if is_locked {
            self.get_lock_ttl(&format!("account_lock:phone:{}", hash_phone(phone))).await?
        } else {
            None
        };
        
        // Get SMS limit status
        let sms_key = format!("rate_limit:sms:{}", hash_phone(phone));
        let sms_count = self.get_current_count(&sms_key).await?;
        
        // Get failed attempts
        let failed_key = format!("failed_attempts:phone:{}", hash_phone(phone));
        let failed_attempts = self.get_current_count(&failed_key).await?;
        
        let limits = vec![
            LimitInfo {
                limit_type: "sms".to_string(),
                current: sms_count,
                limit: self.config.sms.per_phone_per_hour,
                window_seconds: 3600, // 1 hour window
            },
        ];
        
        Ok(RateLimitInfo {
            identifier: phone.to_string(),
            identifier_type: "phone".to_string(),
            is_locked,
            lock_ttl_seconds: lock_ttl,
            limits,
            failed_attempts,
            failed_attempts_threshold: self.config.auth.failed_attempts_threshold,
        })
    }

    /// Get the status of all rate limits for an IP
    pub async fn get_ip_status(&self, ip: &str) -> DomainResult<RateLimitInfo> {
        let is_locked = self.is_ip_locked(ip).await?;
        let lock_ttl = if is_locked {
            self.get_lock_ttl(&format!("account_lock:ip:{}", ip)).await?
        } else {
            None
        };
        
        // Get verification limit status
        let verification_key = format!("rate_limit:ip_verification:{}", ip);
        let verification_count = self.get_current_count(&verification_key).await?;
        
        // Get failed attempts
        let failed_key = format!("failed_attempts:ip:{}", ip);
        let failed_attempts = self.get_current_count(&failed_key).await?;
        
        let limits = vec![
            LimitInfo {
                limit_type: "verification".to_string(),
                current: verification_count,
                limit: self.config.auth.login_per_ip_per_hour,
                window_seconds: 3600,
            },
        ];
        
        Ok(RateLimitInfo {
            identifier: ip.to_string(),
            identifier_type: "ip".to_string(),
            is_locked,
            lock_ttl_seconds: lock_ttl,
            limits,
            failed_attempts,
            failed_attempts_threshold: 10, // Default IP threshold
        })
    }

    /// Reset all limits for a phone number (admin function)
    pub async fn reset_phone_limits(&self, phone: &str) -> DomainResult<()> {
        let mut conn = self.redis_client.get_connection();
        
        let phone_hash = hash_phone(phone);
        let keys = vec![
            format!("rate_limit:sms:{}", phone_hash),
            format!("failed_attempts:phone:{}", phone_hash),
            format!("account_lock:phone:{}", phone_hash),
        ];
        
        for key in keys {
            let _: Result<(), _> = conn.del(&key).await;
        }
        
        Ok(())
    }

    /// Reset all limits for an IP (admin function)
    pub async fn reset_ip_limits(&self, ip: &str) -> DomainResult<()> {
        let mut conn = self.redis_client.get_connection();
        
        let keys = vec![
            format!("rate_limit:ip_verification:{}", ip),
            format!("failed_attempts:ip:{}", ip),
            format!("account_lock:ip:{}", ip),
        ];
        
        for key in keys {
            let _: Result<(), _> = conn.del(&key).await;
        }
        
        Ok(())
    }

    /// Helper to get current count in a sliding window
    async fn get_current_count(&self, key: &str) -> DomainResult<u32> {
        let mut conn = self.redis_client.get_connection();
        
        let now = Utc::now().timestamp_millis();
        let window_start = now - (3600 * 1000); // Default 1 hour window for status
        
        let count: u32 = conn.zcount(key, window_start, "+inf").await
            .unwrap_or(0);
        
        Ok(count)
    }

    /// Helper to get TTL of a lock
    async fn get_lock_ttl(&self, key: &str) -> DomainResult<Option<u64>> {
        let mut conn = self.redis_client.get_connection();
        
        let ttl: i64 = conn.ttl(key).await
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to get TTL: {}", e),
            })?;
        
        if ttl > 0 {
            Ok(Some(ttl as u64))
        } else {
            Ok(None)
        }
    }

    /// Increment failed attempts for a phone and check if should lock
    pub async fn increment_failed_attempts(&self, phone: &str) -> DomainResult<bool> {
        let key = format!("failed_attempts:phone:{}", hash_phone(phone));
        let mut conn = self.redis_client.get_connection();
        
        // Use sliding window for failed attempts too
        let now = Utc::now().timestamp_millis();
        let window_start = now - (3600 * 1000); // 1 hour window
        
        // Remove old entries
        // Remove expired entries using raw Redis command
        let _: redis::RedisResult<i64> = redis::cmd("ZREMRANGEBYSCORE")
            .arg(&key)
            .arg("-inf")
            .arg(window_start)
            .query_async(&mut conn)
            .await;
        
        // Add new failed attempt
        conn.zadd(&key, &now.to_string(), now).await
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to update failed attempts: {}", e),
            })?;
        
        // Count attempts in window
        let count: u32 = conn.zcount(&key, window_start, "+inf").await
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to count failed attempts: {}", e),
            })?;
        
        // Set expiry
        conn.expire(&key, 3600).await
            .map_err(|e| DomainError::Internal {
                message: format!("Failed to set expiry: {}", e),
            })?;
        
        // Check if should lock
        let threshold = self.config.auth.failed_attempts_threshold;
        if count >= threshold {
            self.lock_phone(phone).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[async_trait]
impl RateLimiterTrait for RedisRateLimiter {
    async fn check_sms_rate_limit(&self, phone: &str) -> Result<bool, String> {
        let status = self.check_phone_sms_limit(phone).await
            .map_err(|e| e.to_string())?;
        Ok(!matches!(status, RateLimitStatus::Ok { .. }))
    }

    async fn increment_sms_counter(&self, phone: &str) -> Result<i64, String> {
        let key = format!("rate_limit:sms:{}", hash_phone(phone));
        let mut conn = self.redis_client.get_connection();
        
        let now = Utc::now().timestamp_millis();
        
        // Add to sorted set
        conn.zadd(&key, &now.to_string(), now).await
            .map_err(|e| format!("Failed to increment counter: {}", e))?;
        
        // Set expiry
        let window = 3600i64; // 1 hour window
        conn.expire(&key, window).await
            .map_err(|e| format!("Failed to set expiry: {}", e))?;
        
        // Count entries in window
        let window_start = now - (window as i64 * 1000);
        let count: i64 = conn.zcount(&key, window_start, "+inf").await
            .map_err(|e| format!("Failed to count: {}", e))?;
        
        Ok(count)
    }

    async fn get_rate_limit_reset_time(&self, phone: &str) -> Result<Option<i64>, String> {
        let key = format!("rate_limit:sms:{}", hash_phone(phone));
        let mut conn = self.redis_client.get_connection();
        
        let now = Utc::now().timestamp_millis();
        let window = 3600i64 * 1000; // 1 hour window in milliseconds
        let window_start = now - window;
        
        // Get the oldest entry in the current window
        let oldest: Vec<(String, i64)> = conn.zrangebyscore_limit_withscores(
            &key, 
            window_start, 
            "+inf", 
            0, 
            1
        ).await
        .map_err(|e| format!("Failed to get oldest entry: {}", e))?;
        
        if let Some((_, timestamp)) = oldest.first() {
            let reset_time = (timestamp + window - now) / 1000;
            Ok(Some(reset_time.max(0)))
        } else {
            Ok(None)
        }
    }
}

/// Rate limit status enum
#[derive(Debug, Clone)]
pub enum RateLimitStatus {
    /// Request is within limits
    Ok { 
        remaining: u32,
        limit: u32,
        window_seconds: u64,
    },
    /// Rate limit exceeded
    Exceeded { 
        retry_after_seconds: u64,
        limit: u32,
        window_seconds: u64,
    },
    /// Account/IP is locked
    Locked { 
        retry_after_seconds: u64,
        reason: String,
    },
}

/// Rate limit information for monitoring
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    /// Identifier (phone or IP)
    pub identifier: String,
    /// Type of identifier
    pub identifier_type: String,
    /// Whether the identifier is currently locked
    pub is_locked: bool,
    /// Time until lock expires (if locked)
    pub lock_ttl_seconds: Option<u64>,
    /// Current rate limit statuses
    pub limits: Vec<LimitInfo>,
    /// Number of failed attempts
    pub failed_attempts: u32,
    /// Threshold for locking
    pub failed_attempts_threshold: u32,
}

/// Individual limit information
#[derive(Debug, Clone)]
pub struct LimitInfo {
    /// Type of limit (sms, verification, etc)
    pub limit_type: String,
    /// Current count in window
    pub current: u32,
    /// Maximum allowed
    pub limit: u32,
    /// Window duration in seconds
    pub window_seconds: u64,
}

/// Hash a phone number for audit logging (privacy protection)
fn hash_phone(phone: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(phone.as_bytes());
    format!("{:x}", hasher.finalize())
}