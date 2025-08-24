//! Verification code cache service implementation
//! 
//! This module provides verification code caching functionality with:
//! - 5-minute expiration for verification codes
//! - Attempt tracking with 3 max attempts per code
//! - Secure code storage and validation
//! 
//! The service uses Redis for storage with the following key patterns:
//! - `verification:code:{phone}` - Stores the verification code
//! - `verification:attempts:{phone}` - Tracks verification attempts

use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};

use crate::cache::RedisClient;
use crate::InfrastructureError;

/// Default expiration time for verification codes (5 minutes)
const CODE_EXPIRY_SECONDS: u64 = 300;

/// Maximum allowed verification attempts per code
const MAX_ATTEMPTS: i64 = 3;

/// Expiration time for attempt tracking (same as code expiry)
const ATTEMPTS_EXPIRY_SECONDS: u64 = 300;

/// Verification cache service for managing SMS verification codes
/// 
/// Provides secure storage and validation of verification codes with
/// attempt tracking and automatic expiration.
#[derive(Clone)]
pub struct VerificationCache {
    /// Redis client for cache operations
    redis_client: RedisClient,
}

impl VerificationCache {
    /// Create a new verification cache service
    /// 
    /// # Arguments
    /// * `redis_client` - Redis client for cache operations
    /// 
    /// # Returns
    /// * `VerificationCache` - New verification cache service instance
    /// 
    /// # Example
    /// ```no_run
    /// use renov_infra::cache::{RedisClient, verification_cache::VerificationCache};
    /// 
    /// async fn create_service(redis_client: RedisClient) {
    ///     let service = VerificationCache::new(redis_client);
    /// }
    /// ```
    pub fn new(redis_client: RedisClient) -> Self {
        Self { redis_client }
    }

    /// Store a verification code for a phone number
    /// 
    /// Stores the verification code with a 5-minute expiration and resets
    /// the attempt counter for this phone number.
    /// 
    /// # Arguments
    /// * `phone` - Phone number to associate with the code
    /// * `code` - 6-digit verification code
    /// 
    /// # Returns
    /// * `Result<(), InfrastructureError>` - Success or error
    /// 
    /// # Example
    /// ```no_run
    /// use renov_infra::cache::verification_cache::VerificationCache;
    /// 
    /// async fn store_code(service: &VerificationCache) {
    ///     let phone = "1234567890";
    ///     let code = "123456";
    ///     
    ///     service.store_code(phone, code).await.unwrap();
    /// }
    /// ```
    pub async fn store_code(
        &self,
        phone: &str,
        code: &str,
    ) -> Result<(), InfrastructureError> {
        let code_key = Self::format_code_key(phone);
        let attempts_key = Self::format_attempts_key(phone);
        
        // Hash the code for secure storage
        let hashed_code = Self::hash_code(code);
        
        debug!(
            "Storing verification code for phone: {} (last 4: ...{})",
            Self::mask_phone(phone),
            &phone[phone.len().saturating_sub(4)..]
        );
        
        // Store the hashed code with expiration
        self.redis_client
            .set_with_expiry(&code_key, &hashed_code, CODE_EXPIRY_SECONDS)
            .await?;
        
        // Reset attempt counter (will be created on first verification attempt)
        let _ = self.redis_client.delete(&attempts_key).await;
        
        info!(
            "Verification code stored successfully for phone: {}",
            Self::mask_phone(phone)
        );
        
        Ok(())
    }

    /// Verify a code for a phone number
    /// 
    /// Checks if the provided code matches the stored code for the phone number.
    /// Tracks verification attempts and enforces a maximum of 3 attempts per code.
    /// 
    /// # Arguments
    /// * `phone` - Phone number to verify
    /// * `code` - Verification code to validate
    /// 
    /// # Returns
    /// * `Result<bool, InfrastructureError>` - True if code is valid, false otherwise
    /// 
    /// # Example
    /// ```no_run
    /// use renov_infra::cache::verification_cache::VerificationCache;
    /// 
    /// async fn verify_code(service: &VerificationCache) {
    ///     let phone = "1234567890";
    ///     let code = "123456";
    ///     
    ///     match service.verify_code(phone, code).await {
    ///         Ok(true) => println!("Code verified successfully"),
    ///         Ok(false) => println!("Invalid code or too many attempts"),
    ///         Err(e) => println!("Error: {}", e),
    ///     }
    /// }
    /// ```
    pub async fn verify_code(
        &self,
        phone: &str,
        code: &str,
    ) -> Result<bool, InfrastructureError> {
        let code_key = Self::format_code_key(phone);
        let attempts_key = Self::format_attempts_key(phone);
        
        debug!(
            "Verifying code for phone: {}",
            Self::mask_phone(phone)
        );
        
        // Check current attempt count
        let attempts = self.redis_client
            .increment(&attempts_key, Some(ATTEMPTS_EXPIRY_SECONDS))
            .await?;
        
        // Check if max attempts exceeded
        if attempts > MAX_ATTEMPTS {
            warn!(
                "Maximum verification attempts ({}) exceeded for phone: {}",
                MAX_ATTEMPTS,
                Self::mask_phone(phone)
            );
            return Ok(false);
        }
        
        debug!(
            "Verification attempt {} of {} for phone: {}",
            attempts, MAX_ATTEMPTS,
            Self::mask_phone(phone)
        );
        
        // Retrieve stored hashed code
        let stored_hash = match self.redis_client.get(&code_key).await? {
            Some(hash) => hash,
            None => {
                debug!(
                    "No verification code found for phone: {} (expired or not set)",
                    Self::mask_phone(phone)
                );
                return Ok(false);
            }
        };
        
        // Hash the provided code and compare
        let provided_hash = Self::hash_code(code);
        let is_valid = stored_hash == provided_hash;
        
        if is_valid {
            info!(
                "Verification code validated successfully for phone: {}",
                Self::mask_phone(phone)
            );
            
            // Clean up after successful verification
            let _ = self.redis_client.delete(&code_key).await;
            let _ = self.redis_client.delete(&attempts_key).await;
        } else {
            warn!(
                "Invalid verification code for phone: {} (attempt {}/{})",
                Self::mask_phone(phone),
                attempts,
                MAX_ATTEMPTS
            );
        }
        
        Ok(is_valid)
    }

    /// Get remaining verification attempts for a phone number
    /// 
    /// # Arguments
    /// * `phone` - Phone number to check
    /// 
    /// # Returns
    /// * `Result<i64, InfrastructureError>` - Remaining attempts (0 to MAX_ATTEMPTS)
    /// 
    /// # Example
    /// ```no_run
    /// use renov_infra::cache::verification_cache::VerificationCache;
    /// 
    /// async fn check_attempts(service: &VerificationCache, phone: &str) {
    ///     let remaining = service.get_remaining_attempts(phone).await.unwrap();
    ///     println!("Remaining attempts: {}", remaining);
    /// }
    /// ```
    pub async fn get_remaining_attempts(
        &self,
        phone: &str,
    ) -> Result<i64, InfrastructureError> {
        let attempts_key = Self::format_attempts_key(phone);
        
        // Get current attempts from Redis
        let current_attempts = match self.redis_client.get(&attempts_key).await? {
            Some(count_str) => count_str.parse::<i64>().unwrap_or(0),
            None => 0,
        };
        
        let remaining = (MAX_ATTEMPTS - current_attempts).max(0);
        
        debug!(
            "Remaining attempts for phone {}: {}",
            Self::mask_phone(phone),
            remaining
        );
        
        Ok(remaining)
    }

    /// Check if a verification code exists for a phone number
    /// 
    /// # Arguments
    /// * `phone` - Phone number to check
    /// 
    /// # Returns
    /// * `Result<bool, InfrastructureError>` - True if code exists and hasn't expired
    /// 
    /// # Example
    /// ```no_run
    /// use renov_infra::cache::verification_cache::VerificationCache;
    /// 
    /// async fn check_code_exists(service: &VerificationCache, phone: &str) {
    ///     if service.code_exists(phone).await.unwrap() {
    ///         println!("Code exists for phone");
    ///     }
    /// }
    /// ```
    pub async fn code_exists(&self, phone: &str) -> Result<bool, InfrastructureError> {
        let code_key = Self::format_code_key(phone);
        self.redis_client.exists(&code_key).await
    }

    /// Get time-to-live for a verification code
    /// 
    /// # Arguments
    /// * `phone` - Phone number to check
    /// 
    /// # Returns
    /// * `Result<Option<i64>, InfrastructureError>` - TTL in seconds, None if code doesn't exist
    /// 
    /// # Example
    /// ```no_run
    /// use renov_infra::cache::verification_cache::VerificationCache;
    /// 
    /// async fn check_ttl(service: &VerificationCache, phone: &str) {
    ///     if let Some(ttl) = service.get_code_ttl(phone).await.unwrap() {
    ///         println!("Code expires in {} seconds", ttl);
    ///     }
    /// }
    /// ```
    pub async fn get_code_ttl(&self, phone: &str) -> Result<Option<i64>, InfrastructureError> {
        let code_key = Self::format_code_key(phone);
        self.redis_client.ttl(&code_key).await
    }

    /// Clear verification code and attempts for a phone number
    /// 
    /// Useful for cleanup or when a user requests a new code.
    /// 
    /// # Arguments
    /// * `phone` - Phone number to clear
    /// 
    /// # Returns
    /// * `Result<(), InfrastructureError>` - Success or error
    /// 
    /// # Example
    /// ```no_run
    /// use renov_infra::cache::verification_cache::VerificationCache;
    /// 
    /// async fn clear_code(service: &VerificationCache, phone: &str) {
    ///     service.clear_verification(phone).await.unwrap();
    /// }
    /// ```
    pub async fn clear_verification(&self, phone: &str) -> Result<(), InfrastructureError> {
        let code_key = Self::format_code_key(phone);
        let attempts_key = Self::format_attempts_key(phone);
        
        debug!(
            "Clearing verification data for phone: {}",
            Self::mask_phone(phone)
        );
        
        let _ = self.redis_client.delete(&code_key).await;
        let _ = self.redis_client.delete(&attempts_key).await;
        
        info!(
            "Verification data cleared for phone: {}",
            Self::mask_phone(phone)
        );
        
        Ok(())
    }

    /// Format Redis key for verification code storage
    fn format_code_key(phone: &str) -> String {
        format!("verification:code:{}", phone)
    }

    /// Format Redis key for attempt tracking
    fn format_attempts_key(phone: &str) -> String {
        format!("verification:attempts:{}", phone)
    }

    /// Hash a verification code using SHA-256
    /// 
    /// Provides secure storage by hashing codes before storing in Redis.
    fn hash_code(code: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(code.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    /// Mask phone number for logging (show only last 4 digits)
    /// 
    /// Implements security requirement to desensitize phone numbers in logs.
    fn mask_phone(phone: &str) -> String {
        if phone.len() <= 4 {
            "****".to_string()
        } else {
            format!("***{}", &phone[phone.len() - 4..])
        }
    }
}