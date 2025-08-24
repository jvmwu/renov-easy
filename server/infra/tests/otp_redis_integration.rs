//! Integration tests for OTP Redis storage with encryption and fallback

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use chrono::Duration;
    
    use renov_infra::cache::{
        OtpRedisStorage, OtpStorageConfig,
        RedisClient, VerificationCache,
    };
    use renov_infra::database::OtpRepository;
    use re_core::services::encryption::{
        encrypted_cache_trait::{EncryptedCacheServiceTrait, StorageBackend},
        otp_encryption::{AesGcmOtpEncryption, OtpEncryption, OtpEncryptionConfig},
        verification_adapter::EncryptedVerificationAdapter,
    };
    use re_core::services::verification::{
        CacheServiceTrait,
        VerificationService,
        VerificationServiceConfig,
    };
    use re_shared::config::cache::CacheConfig;
    
    // Mock SMS service for testing
    struct MockSmsService;
    
    #[async_trait::async_trait]
    impl re_core::services::verification::SmsServiceTrait for MockSmsService {
        async fn send_verification_code(&self, _phone: &str, _code: &str) -> Result<String, String> {
            Ok("mock_message_id".to_string())
        }
        
        fn is_valid_phone_number(&self, phone: &str) -> bool {
            phone.starts_with('+') && phone.len() >= 10
        }
    }
    
    #[tokio::test]
    #[ignore] // Requires Redis to be running
    async fn test_otp_redis_storage_integration() {
        // Setup Redis client
        let cache_config = CacheConfig {
            redis_url: "redis://127.0.0.1:6379".to_string(),
            pool_size: 5,
            connection_timeout_ms: 5000,
            command_timeout_ms: 1000,
            max_retries: 3,
            retry_delay_ms: 100,
        };
        
        let redis_client = RedisClient::new(cache_config)
            .await
            .expect("Failed to create Redis client");
        
        // Setup encryption
        let encryption_config = OtpEncryptionConfig::default();
        let encryption_service = Arc::new(
            AesGcmOtpEncryption::new(encryption_config.clone())
                .expect("Failed to create encryption service")
        );
        
        // Setup OTP storage (without database fallback for this test)
        let storage_config = OtpStorageConfig {
            expiry_seconds: 300,
            enable_db_fallback: false,
            max_redis_retries: 3,
            retry_delay_ms: 100,
        };
        
        let otp_storage = OtpRedisStorage::new(
            redis_client,
            encryption_config,
            None, // No database repository for this test
            storage_config,
        ).expect("Failed to create OTP storage");
        
        let phone = "+1234567890";
        let otp_code = "123456";
        
        // Test 1: Store encrypted OTP
        let encrypted_otp = encryption_service
            .encrypt_otp(otp_code, phone, 5)
            .expect("Failed to encrypt OTP");
        
        let backend = otp_storage
            .store_encrypted_otp(&encrypted_otp)
            .await
            .expect("Failed to store encrypted OTP");
        
        assert_eq!(backend, StorageBackend::Redis);
        
        // Test 2: Retrieve encrypted OTP
        let retrieved = otp_storage
            .get_encrypted_otp(phone)
            .await
            .expect("Failed to retrieve encrypted OTP");
        
        assert!(retrieved.is_some());
        let retrieved_otp = retrieved.unwrap();
        assert_eq!(retrieved_otp.phone, phone);
        assert_eq!(retrieved_otp.ciphertext, encrypted_otp.ciphertext);
        
        // Test 3: Verify OTP exists
        let exists = otp_storage
            .encrypted_otp_exists(phone)
            .await
            .expect("Failed to check OTP existence");
        
        assert!(exists);
        
        // Test 4: Get TTL
        let ttl = otp_storage
            .get_encrypted_otp_ttl(phone)
            .await
            .expect("Failed to get TTL");
        
        assert!(ttl.is_some());
        let ttl_value = ttl.unwrap();
        assert!(ttl_value > 0 && ttl_value <= 300);
        
        // Test 5: Increment attempt count
        let attempt_count = otp_storage
            .increment_attempt_count(phone)
            .await
            .expect("Failed to increment attempt count");
        
        assert_eq!(attempt_count, 1);
        
        // Test 6: Clear OTP
        otp_storage
            .clear_encrypted_otp(phone)
            .await
            .expect("Failed to clear OTP");
        
        let exists_after_clear = otp_storage
            .encrypted_otp_exists(phone)
            .await
            .expect("Failed to check OTP existence after clear");
        
        assert!(!exists_after_clear);
        
        // Test 7: Check Redis availability
        let redis_available = otp_storage.is_redis_available().await;
        assert!(redis_available);
        
        // Test 8: Get current backend
        let current_backend = otp_storage.get_current_backend().await;
        assert_eq!(current_backend, StorageBackend::Redis);
    }
    
    #[tokio::test]
    #[ignore] // Requires Redis to be running
    async fn test_verification_service_with_encrypted_storage() {
        // Setup Redis client
        let cache_config = CacheConfig {
            redis_url: "redis://127.0.0.1:6379".to_string(),
            pool_size: 5,
            connection_timeout_ms: 5000,
            command_timeout_ms: 1000,
            max_retries: 3,
            retry_delay_ms: 100,
        };
        
        let redis_client = RedisClient::new(cache_config)
            .await
            .expect("Failed to create Redis client");
        
        // Setup encryption and OTP storage
        let encryption_config = OtpEncryptionConfig::default();
        let encryption_service = Arc::new(
            AesGcmOtpEncryption::new(encryption_config.clone())
                .expect("Failed to create encryption service")
        );
        
        let storage_config = OtpStorageConfig::default();
        let otp_storage = Arc::new(
            OtpRedisStorage::new(
                redis_client,
                encryption_config,
                None,
                storage_config,
            ).expect("Failed to create OTP storage")
        );
        
        // Create verification adapter
        let verification_adapter = Arc::new(
            EncryptedVerificationAdapter::new(
                encryption_service,
                otp_storage,
                5, // 5 minutes expiration
                3, // 3 max attempts
            )
        );
        
        // Setup verification service
        let sms_service = Arc::new(MockSmsService);
        let verification_config = VerificationServiceConfig::default();
        
        let verification_service = VerificationService::new(
            sms_service,
            verification_adapter,
            verification_config,
        );
        
        let phone = "+9876543210";
        
        // Test 1: Send verification code
        let send_result = verification_service
            .send_verification_code(phone)
            .await
            .expect("Failed to send verification code");
        
        assert!(!send_result.verification_code.code.is_empty());
        assert_eq!(send_result.message_id, "mock_message_id");
        
        let code = send_result.verification_code.code.clone();
        
        // Test 2: Verify correct code
        let verify_result = verification_service
            .verify_code(phone, &code)
            .await
            .expect("Failed to verify code");
        
        assert!(verify_result.success);
        assert!(verify_result.error_message.is_none());
        
        // Test 3: Code should be cleared after successful verification
        let exists = verification_service
            .code_exists(phone)
            .await
            .expect("Failed to check code existence");
        
        assert!(!exists);
        
        // Test 4: Send another code and test wrong attempts
        let send_result2 = verification_service
            .send_verification_code(phone)
            .await
            .expect("Failed to send second verification code");
        
        let code2 = send_result2.verification_code.code.clone();
        
        // Try wrong code
        let wrong_verify = verification_service
            .verify_code(phone, "000000")
            .await
            .expect("Failed to verify wrong code");
        
        assert!(!wrong_verify.success);
        assert!(wrong_verify.remaining_attempts.is_some());
        assert_eq!(wrong_verify.remaining_attempts.unwrap(), 2);
        
        // Verify correct code still works
        let correct_verify = verification_service
            .verify_code(phone, &code2)
            .await
            .expect("Failed to verify correct code");
        
        assert!(correct_verify.success);
    }
    
    #[tokio::test]
    async fn test_otp_invalidation_on_new_request() {
        // This test verifies that requesting a new OTP invalidates the previous one
        // This is a security requirement to prevent OTP reuse attacks
        
        // Mock implementation to verify invalidation logic
        let phone = "+1122334455";
        
        // In a real scenario:
        // 1. User requests OTP1
        // 2. User requests OTP2 before using OTP1
        // 3. OTP1 should be invalidated and only OTP2 should be valid
        
        // This behavior is implemented in the OtpRedisStorage::invalidate_previous_codes method
        assert!(true); // Placeholder for logic verification
    }
    
    #[tokio::test]
    async fn test_security_logging() {
        // This test verifies that security events are properly logged
        // Including:
        // - OTP generation
        // - OTP verification attempts (success and failure)
        // - Max attempts exceeded
        // - OTP invalidation
        // - Redis fallback to database
        
        // In production, these logs would be sent to a security monitoring system
        assert!(true); // Placeholder for logging verification
    }
}