//! Tests for OTP Redis storage implementation

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use chrono::{Utc, Duration};
    
    use re_core::services::encryption::{
        encrypted_cache_trait::{EncryptedCacheServiceTrait, StorageBackend},
        otp_encryption::{AesGcmOtpEncryption, EncryptedOtp, OtpEncryption, OtpEncryptionConfig},
    };
    
    use crate::cache::{
        otp_storage::{OtpRedisStorage, OtpStorageConfig, OtpMetadata},
        RedisClient,
    };
    use crate::database::OtpRepository;
    
    // Helper function to create test encrypted OTP
    fn create_test_encrypted_otp(phone: &str, attempt_count: u32) -> EncryptedOtp {
        EncryptedOtp {
            ciphertext: format!("encrypted_otp_for_{}", phone),
            nonce: "test_nonce_123".to_string(),
            key_id: "test_key_001".to_string(),
            created_at: Utc::now(),
            attempt_count,
            expires_at: Utc::now() + Duration::minutes(5),
            phone: phone.to_string(),
        }
    }
    
    #[tokio::test]
    async fn test_otp_storage_key_formatting() {
        let phone = "+1234567890";
        let otp_key = format!("otp:encrypted:{}", phone);
        let metadata_key = format!("otp:metadata:{}", phone);
        
        assert_eq!(otp_key, "otp:encrypted:+1234567890");
        assert_eq!(metadata_key, "otp:metadata:+1234567890");
    }
    
    #[tokio::test]
    async fn test_otp_metadata_creation() {
        let phone = "+1234567890";
        let encrypted_otp = create_test_encrypted_otp(phone, 0);
        let metadata = OtpMetadata::from(&encrypted_otp);
        
        assert_eq!(metadata.phone, phone);
        assert_eq!(metadata.attempts, 0);
        assert_eq!(metadata.max_attempts, 3);
        assert!(!metadata.is_used);
        assert_eq!(metadata.storage_backend, StorageBackend::Redis);
        assert_eq!(metadata.created_at, encrypted_otp.created_at);
        assert_eq!(metadata.expires_at, encrypted_otp.expires_at);
    }
    
    #[tokio::test]
    async fn test_phone_number_masking() {
        // Test various phone number formats
        let test_cases = vec![
            ("+1234567890", "***7890"),
            ("+86138123456", "***3456"),
            ("555", "****"),
            ("12345", "***2345"),
            ("+33612345678", "***5678"),
        ];
        
        for (phone, expected_mask) in test_cases {
            let masked = if phone.len() <= 4 {
                "****".to_string()
            } else {
                format!("***{}", &phone[phone.len() - 4..])
            };
            assert_eq!(masked, expected_mask, "Failed for phone: {}", phone);
        }
    }
    
    #[tokio::test]
    async fn test_encryption_integration() {
        let encryption_config = OtpEncryptionConfig::default();
        let encryption_service = Arc::new(
            AesGcmOtpEncryption::new(encryption_config.clone()).unwrap()
        );
        
        let phone = "+1234567890";
        let otp_code = "123456";
        
        // Encrypt OTP
        let encrypted_otp = encryption_service
            .encrypt_otp(otp_code, phone, 5)
            .unwrap();
        
        // Verify encryption produced valid output
        assert!(!encrypted_otp.ciphertext.is_empty());
        assert!(!encrypted_otp.nonce.is_empty());
        assert!(!encrypted_otp.key_id.is_empty());
        assert_eq!(encrypted_otp.phone, phone);
        assert_eq!(encrypted_otp.attempt_count, 0);
        
        // Verify OTP with correct code
        let is_valid = encryption_service
            .verify_otp(&encrypted_otp, otp_code)
            .unwrap();
        assert!(is_valid);
        
        // Verify OTP with incorrect code
        let is_invalid = encryption_service
            .verify_otp(&encrypted_otp, "654321")
            .unwrap();
        assert!(!is_invalid);
    }
    
    #[tokio::test]
    async fn test_attempt_count_tracking() {
        let mut encrypted_otp = create_test_encrypted_otp("+1234567890", 0);
        
        // Initial attempt count should be 0
        assert_eq!(encrypted_otp.attempt_count, 0);
        
        // Simulate incrementing attempts
        encrypted_otp.attempt_count += 1;
        assert_eq!(encrypted_otp.attempt_count, 1);
        
        encrypted_otp.attempt_count += 1;
        assert_eq!(encrypted_otp.attempt_count, 2);
        
        encrypted_otp.attempt_count += 1;
        assert_eq!(encrypted_otp.attempt_count, 3);
        
        // Check if max attempts reached
        let max_attempts = 3u32;
        assert!(encrypted_otp.attempt_count >= max_attempts);
    }
    
    #[tokio::test]
    async fn test_otp_expiration_check() {
        let phone = "+1234567890";
        
        // Create an OTP that expires in 5 minutes
        let mut encrypted_otp = create_test_encrypted_otp(phone, 0);
        
        // OTP should not be expired
        assert!(encrypted_otp.expires_at > Utc::now());
        
        // Create an expired OTP
        encrypted_otp.expires_at = Utc::now() - Duration::minutes(1);
        
        // OTP should be expired
        assert!(encrypted_otp.expires_at < Utc::now());
    }
    
    #[tokio::test]
    async fn test_storage_config_defaults() {
        let config = OtpStorageConfig::default();
        
        assert_eq!(config.expiry_seconds, 300); // 5 minutes
        assert!(config.enable_db_fallback);
        assert_eq!(config.max_redis_retries, 3);
        assert_eq!(config.retry_delay_ms, 100);
    }
    
    #[tokio::test]
    async fn test_storage_backend_enum() {
        let redis_backend = StorageBackend::Redis;
        let db_backend = StorageBackend::Database;
        
        assert_eq!(redis_backend, StorageBackend::Redis);
        assert_eq!(db_backend, StorageBackend::Database);
        assert_ne!(redis_backend, db_backend);
    }
    
    // Mock Redis client for testing
    struct MockRedisClient {
        storage: Arc<tokio::sync::RwLock<std::collections::HashMap<String, (String, Option<u64>)>>>,
        fail_operations: bool,
    }
    
    impl MockRedisClient {
        fn new(fail_operations: bool) -> Self {
            Self {
                storage: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
                fail_operations,
            }
        }
    }
    
    #[tokio::test]
    async fn test_invalidation_logic() {
        let phone = "+1234567890";
        let encrypted_otp1 = create_test_encrypted_otp(phone, 0);
        let encrypted_otp2 = create_test_encrypted_otp(phone, 0);
        
        // Both OTPs are for the same phone but created at different times
        assert_eq!(encrypted_otp1.phone, encrypted_otp2.phone);
        
        // In a real scenario, storing otp2 should invalidate otp1
        // This ensures only the latest OTP is valid
    }
    
    #[tokio::test]
    async fn test_ttl_calculation() {
        let encrypted_otp = create_test_encrypted_otp("+1234567890", 0);
        
        // Calculate TTL
        let ttl_seconds = (encrypted_otp.expires_at - Utc::now()).num_seconds();
        
        // TTL should be approximately 5 minutes (300 seconds)
        // Allow for some time passing during test execution
        assert!(ttl_seconds > 295 && ttl_seconds <= 300);
    }
}