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
    /// use renov_infrastructure::cache::{RedisClient, verification_cache::VerificationCache};
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
    /// use renov_infrastructure::cache::verification_cache::VerificationCache;
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
    /// use renov_infrastructure::cache::verification_cache::VerificationCache;
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
    /// use renov_infrastructure::cache::verification_cache::VerificationCache;
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
    /// use renov_infrastructure::cache::verification_cache::VerificationCache;
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
    /// use renov_infrastructure::cache::verification_cache::VerificationCache;
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
    /// use renov_infrastructure::cache::verification_cache::VerificationCache;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::CacheConfig;

    #[test]
    fn test_format_keys() {
        let phone = "1234567890";
        
        assert_eq!(
            VerificationCache::format_code_key(phone),
            "verification:code:1234567890"
        );
        
        assert_eq!(
            VerificationCache::format_attempts_key(phone),
            "verification:attempts:1234567890"
        );
    }

    #[test]
    fn test_hash_code() {
        let code1 = "123456";
        let code2 = "654321";
        let code1_duplicate = "123456";
        
        let hash1 = VerificationCache::hash_code(code1);
        let hash2 = VerificationCache::hash_code(code2);
        let hash1_dup = VerificationCache::hash_code(code1_duplicate);
        
        // Same code should produce same hash
        assert_eq!(hash1, hash1_dup);
        
        // Different codes should produce different hashes
        assert_ne!(hash1, hash2);
        
        // Hash should be hex string (64 chars for SHA-256)
        assert_eq!(hash1.len(), 64);
        assert!(hash1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_mask_phone() {
        assert_eq!(VerificationCache::mask_phone("1234567890"), "***7890");
        assert_eq!(VerificationCache::mask_phone("567890"), "***7890");
        assert_eq!(VerificationCache::mask_phone("1234"), "****");
        assert_eq!(VerificationCache::mask_phone("123"), "****");
        assert_eq!(VerificationCache::mask_phone(""), "****");
    }

    #[tokio::test]
    #[ignore] // Requires actual Redis server
    async fn test_store_and_verify_code() {
        let config = CacheConfig {
            url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            pool_size: 5,
            default_ttl: 3600,
        };

        let redis_client = RedisClient::new(config).await.unwrap();
        let service = VerificationCache::new(redis_client);
        
        let phone = "test_1234567890";
        let code = "123456";
        
        // Clean up from previous tests
        service.clear_verification(phone).await.unwrap();
        
        // Store code
        service.store_code(phone, code).await.unwrap();
        
        // Verify correct code
        let valid = service.verify_code(phone, code).await.unwrap();
        assert!(valid);
        
        // Code should be deleted after successful verification
        let exists = service.code_exists(phone).await.unwrap();
        assert!(!exists);
    }

    #[tokio::test]
    #[ignore] // Requires actual Redis server
    async fn test_verify_incorrect_code() {
        let config = CacheConfig {
            url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            pool_size: 5,
            default_ttl: 3600,
        };

        let redis_client = RedisClient::new(config).await.unwrap();
        let service = VerificationCache::new(redis_client);
        
        let phone = "test_9876543210";
        let correct_code = "123456";
        let wrong_code = "654321";
        
        // Clean up from previous tests
        service.clear_verification(phone).await.unwrap();
        
        // Store code
        service.store_code(phone, correct_code).await.unwrap();
        
        // Verify incorrect code
        let valid = service.verify_code(phone, wrong_code).await.unwrap();
        assert!(!valid);
        
        // Code should still exist after failed attempt
        let exists = service.code_exists(phone).await.unwrap();
        assert!(exists);
        
        // Should have 2 remaining attempts
        let remaining = service.get_remaining_attempts(phone).await.unwrap();
        assert_eq!(remaining, 2);
    }

    #[tokio::test]
    #[ignore] // Requires actual Redis server
    async fn test_max_attempts() {
        let config = CacheConfig {
            url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            pool_size: 5,
            default_ttl: 3600,
        };

        let redis_client = RedisClient::new(config).await.unwrap();
        let service = VerificationCache::new(redis_client);
        
        let phone = "test_5555555555";
        let correct_code = "123456";
        let wrong_code = "000000";
        
        // Clean up from previous tests
        service.clear_verification(phone).await.unwrap();
        
        // Store code
        service.store_code(phone, correct_code).await.unwrap();
        
        // Make 3 failed attempts
        for i in 1..=3 {
            let valid = service.verify_code(phone, wrong_code).await.unwrap();
            assert!(!valid, "Attempt {} should fail", i);
        }
        
        // 4th attempt should fail even with correct code (max attempts exceeded)
        let valid = service.verify_code(phone, correct_code).await.unwrap();
        assert!(!valid, "Should fail due to max attempts");
        
        // Remaining attempts should be 0
        let remaining = service.get_remaining_attempts(phone).await.unwrap();
        assert_eq!(remaining, 0);
    }

    #[tokio::test]
    #[ignore] // Requires actual Redis server
    async fn test_code_expiry() {
        let config = CacheConfig {
            url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            pool_size: 5,
            default_ttl: 3600,
        };

        let redis_client = RedisClient::new(config).await.unwrap();
        let service = VerificationCache::new(redis_client);
        
        let phone = "test_expiry_check";
        let code = "999999";
        
        // Clean up from previous tests
        service.clear_verification(phone).await.unwrap();
        
        // Store code
        service.store_code(phone, code).await.unwrap();
        
        // Check TTL
        let ttl = service.get_code_ttl(phone).await.unwrap();
        assert!(ttl.is_some());
        let ttl_value = ttl.unwrap();
        assert!(ttl_value > 0 && ttl_value <= CODE_EXPIRY_SECONDS as i64);
        
        // Clean up
        service.clear_verification(phone).await.unwrap();
    }
}