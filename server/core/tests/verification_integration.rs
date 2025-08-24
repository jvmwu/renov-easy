//! Integration tests for verification service with enhanced security

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use async_trait::async_trait;
    use chrono::Utc;
    
    use re_core::services::verification::{
        CacheServiceTrait,
        SmsServiceTrait,
        VerificationService,
        VerificationServiceConfig,
    };
    
    // Mock SMS service
    struct MockSmsService {
        send_success: bool,
    }
    
    impl MockSmsService {
        fn new(send_success: bool) -> Self {
            Self { send_success }
        }
    }
    
    #[async_trait]
    impl SmsServiceTrait for MockSmsService {
        async fn send_verification_code(&self, phone: &str, code: &str) -> Result<String, String> {
            if self.send_success {
                Ok(format!("msg_id_{}", Utc::now().timestamp()))
            } else {
                Err("SMS sending failed".to_string())
            }
        }
        
        fn is_valid_phone_number(&self, phone: &str) -> bool {
            phone.starts_with('+') && phone.len() >= 10
        }
    }
    
    // Mock cache service with attempt tracking
    struct MockCacheService {
        codes: Arc<tokio::sync::RwLock<std::collections::HashMap<String, (String, u32)>>>,
        max_attempts: u32,
    }
    
    impl MockCacheService {
        fn new() -> Self {
            Self {
                codes: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
                max_attempts: 3,
            }
        }
    }
    
    #[async_trait]
    impl CacheServiceTrait for MockCacheService {
        async fn store_code(&self, phone: &str, code: &str) -> Result<(), String> {
            let mut codes = self.codes.write().await;
            codes.insert(phone.to_string(), (code.to_string(), 0));
            Ok(())
        }
        
        async fn verify_code(&self, phone: &str, code: &str) -> Result<bool, String> {
            let mut codes = self.codes.write().await;
            
            if let Some((stored_code, attempts)) = codes.get_mut(phone) {
                *attempts += 1;
                
                if *attempts > self.max_attempts {
                    codes.remove(phone);
                    return Ok(false);
                }
                
                if stored_code == code {
                    codes.remove(phone);
                    return Ok(true);
                }
                
                Ok(false)
            } else {
                Ok(false)
            }
        }
        
        async fn get_remaining_attempts(&self, phone: &str) -> Result<i64, String> {
            let codes = self.codes.read().await;
            
            if let Some((_, attempts)) = codes.get(phone) {
                Ok((self.max_attempts - attempts) as i64)
            } else {
                Ok(0)
            }
        }
        
        async fn code_exists(&self, phone: &str) -> Result<bool, String> {
            let codes = self.codes.read().await;
            Ok(codes.contains_key(phone))
        }
        
        async fn get_code_ttl(&self, phone: &str) -> Result<Option<i64>, String> {
            let codes = self.codes.read().await;
            if codes.contains_key(phone) {
                Ok(Some(300)) // 5 minutes
            } else {
                Ok(None)
            }
        }
        
        async fn clear_verification(&self, phone: &str) -> Result<(), String> {
            let mut codes = self.codes.write().await;
            codes.remove(phone);
            Ok(())
        }
    }
    
    #[tokio::test]
    async fn test_complete_verification_flow_with_security() {
        let sms_service = Arc::new(MockSmsService::new(true));
        let cache_service = Arc::new(MockCacheService::new());
        let config = VerificationServiceConfig::default();
        
        let verification_service = VerificationService::new(
            sms_service,
            cache_service,
            config,
        );
        
        let phone = "+1234567890";
        
        // Step 1: Send verification code
        let send_result = verification_service
            .send_verification_code(phone)
            .await
            .unwrap();
        
        assert!(!send_result.verification_code.code.is_empty());
        assert_eq!(send_result.verification_code.code.len(), 6);
        
        let code = send_result.verification_code.code.clone();
        
        // Step 2: Try wrong code twice
        let wrong_code = "000000";
        
        // First wrong attempt
        let verify_result1 = verification_service
            .verify_code(phone, wrong_code)
            .await
            .unwrap();
        
        assert!(!verify_result1.success);
        assert_eq!(verify_result1.remaining_attempts, Some(2));
        
        // Second wrong attempt
        let verify_result2 = verification_service
            .verify_code(phone, wrong_code)
            .await
            .unwrap();
        
        assert!(!verify_result2.success);
        assert_eq!(verify_result2.remaining_attempts, Some(1));
        
        // Step 3: Verify with correct code
        let verify_result3 = verification_service
            .verify_code(phone, &code)
            .await
            .unwrap();
        
        assert!(verify_result3.success);
        assert!(verify_result3.error_message.is_none());
        
        // Step 4: Code should be cleared after successful verification
        let exists = verification_service
            .code_exists(phone)
            .await
            .unwrap();
        
        assert!(!exists);
    }
    
    #[tokio::test]
    async fn test_max_attempts_lockout() {
        let sms_service = Arc::new(MockSmsService::new(true));
        let cache_service = Arc::new(MockCacheService::new());
        let config = VerificationServiceConfig::default();
        
        let verification_service = VerificationService::new(
            sms_service,
            cache_service,
            config,
        );
        
        let phone = "+9876543210";
        
        // Send verification code
        let send_result = verification_service
            .send_verification_code(phone)
            .await
            .unwrap();
        
        let wrong_code = "111111";
        
        // Make 3 wrong attempts
        for i in 0..3 {
            let verify_result = verification_service
                .verify_code(phone, wrong_code)
                .await
                .unwrap();
            
            assert!(!verify_result.success);
            
            if i < 2 {
                assert_eq!(verify_result.remaining_attempts, Some((2 - i) as i32));
            } else {
                // After 3rd attempt, account should be locked
                assert_eq!(verify_result.remaining_attempts, Some(0));
                assert!(verify_result.error_message.unwrap().contains("Maximum verification attempts exceeded"));
            }
        }
        
        // Even correct code should fail after max attempts
        let correct_code = send_result.verification_code.code;
        let verify_result = verification_service
            .verify_code(phone, &correct_code)
            .await
            .unwrap();
        
        assert!(!verify_result.success);
    }
    
    #[tokio::test]
    async fn test_rate_limiting_on_resend() {
        let sms_service = Arc::new(MockSmsService::new(true));
        let cache_service = Arc::new(MockCacheService::new());
        
        let mut config = VerificationServiceConfig::default();
        config.resend_cooldown_seconds = 60; // 60 seconds cooldown
        
        let verification_service = VerificationService::new(
            sms_service,
            cache_service,
            config,
        );
        
        let phone = "+5551234567";
        
        // Send first code
        let _ = verification_service
            .send_verification_code(phone)
            .await
            .unwrap();
        
        // Try to send another code immediately - should fail due to cooldown
        let resend_result = verification_service
            .send_verification_code(phone)
            .await;
        
        assert!(resend_result.is_err());
        
        if let Err(e) = resend_result {
            let error_str = format!("{:?}", e);
            assert!(error_str.contains("RateLimitExceeded") || error_str.contains("cooldown"));
        }
    }
    
    #[tokio::test]
    async fn test_account_lock_check() {
        let sms_service = Arc::new(MockSmsService::new(true));
        let cache_service = Arc::new(MockCacheService::new());
        let config = VerificationServiceConfig::default();
        
        let verification_service = VerificationService::new(
            sms_service,
            cache_service,
            config,
        );
        
        let phone = "+3331112222";
        
        // Initially account should not be locked
        let is_locked = verification_service
            .is_account_locked(phone)
            .await
            .unwrap();
        
        assert!(!is_locked);
        
        // After implementing actual lock storage, this would test:
        // 1. Lock account after max attempts
        // 2. Check that is_account_locked returns true
        // 3. Unlock account
        // 4. Check that is_account_locked returns false
    }
    
    #[tokio::test]
    async fn test_verification_stats_retrieval() {
        let sms_service = Arc::new(MockSmsService::new(true));
        let cache_service = Arc::new(MockCacheService::new());
        let config = VerificationServiceConfig::default();
        
        let verification_service = VerificationService::new(
            sms_service,
            cache_service,
            config,
        );
        
        let phone = "+7778889999";
        
        // Get initial stats
        let stats = verification_service
            .get_verification_stats(phone)
            .await
            .unwrap();
        
        assert_eq!(stats.total_attempts, 0);
        assert_eq!(stats.successful_verifications, 0);
        assert_eq!(stats.failed_verifications, 0);
        assert_eq!(stats.account_locks, 0);
        assert!(stats.last_attempt.is_none());
        assert!(stats.last_successful.is_none());
        
        // After implementing actual stats tracking, this would test:
        // 1. Send code and verify successfully
        // 2. Check stats show 1 successful verification
        // 3. Make failed attempts
        // 4. Check stats show failed attempts
    }
}