//! Unit tests for Twilio SMS service

#[cfg(test)]
#[cfg(feature = "twilio-sms")]
mod tests {
    use crate::sms::{TwilioConfig, TwilioSmsService, TwilioSmsServiceAdapter};
    use crate::sms::sms_service::SmsService;
    use re_core::services::verification::SmsServiceTrait;
    
    fn setup_test_config() -> TwilioConfig {
        TwilioConfig {
            account_sid: "ACtest_account_sid".to_string(),
            auth_token: "test_auth_token".to_string(),
            from_number: "+15551234567".to_string(),
            max_retries: 3,
            retry_delay_ms: 100,
            request_timeout_secs: 10,
        }
    }
    
    fn setup_test_env() {
        std::env::set_var("TWILIO_ACCOUNT_SID", "ACtest_account_sid");
        std::env::set_var("TWILIO_AUTH_TOKEN", "test_auth_token");
        std::env::set_var("TWILIO_FROM_NUMBER", "+15551234567");
        std::env::set_var("TWILIO_MAX_RETRIES", "2");
        std::env::set_var("TWILIO_RETRY_DELAY_MS", "50");
        std::env::set_var("TWILIO_REQUEST_TIMEOUT_SECS", "5");
    }
    
    fn cleanup_test_env() {
        std::env::remove_var("TWILIO_ACCOUNT_SID");
        std::env::remove_var("TWILIO_AUTH_TOKEN");
        std::env::remove_var("TWILIO_FROM_NUMBER");
        std::env::remove_var("TWILIO_MAX_RETRIES");
        std::env::remove_var("TWILIO_RETRY_DELAY_MS");
        std::env::remove_var("TWILIO_REQUEST_TIMEOUT_SECS");
    }
    
    #[test]
    fn test_twilio_config_creation() {
        let config = setup_test_config();
        assert_eq!(config.account_sid, "ACtest_account_sid");
        assert_eq!(config.auth_token, "test_auth_token");
        assert_eq!(config.from_number, "+15551234567");
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay_ms, 100);
        assert_eq!(config.request_timeout_secs, 10);
    }
    
    #[test]
    fn test_twilio_config_from_env() {
        setup_test_env();
        
        let config = TwilioConfig::from_env().expect("Should create config from env");
        assert_eq!(config.account_sid, "ACtest_account_sid");
        assert_eq!(config.auth_token, "test_auth_token");
        assert_eq!(config.from_number, "+15551234567");
        assert_eq!(config.max_retries, 2);
        assert_eq!(config.retry_delay_ms, 50);
        assert_eq!(config.request_timeout_secs, 5);
        
        cleanup_test_env();
    }
    
    #[test]
    fn test_twilio_config_env_validation() {
        // Clean up first - be more thorough
        std::env::remove_var("TWILIO_ACCOUNT_SID");
        std::env::remove_var("TWILIO_AUTH_TOKEN");
        std::env::remove_var("TWILIO_FROM_NUMBER");
        std::env::remove_var("TWILIO_MAX_RETRIES");
        std::env::remove_var("TWILIO_RETRY_DELAY_MS");
        std::env::remove_var("TWILIO_REQUEST_TIMEOUT_SECS");
        
        // Test missing account SID
        std::env::set_var("TWILIO_AUTH_TOKEN", "test_token");
        std::env::set_var("TWILIO_FROM_NUMBER", "+15551234567");
        
        let result = TwilioConfig::from_env();
        assert!(result.is_err(), "Should fail when TWILIO_ACCOUNT_SID is missing");
        
        // Clean up before next test
        std::env::remove_var("TWILIO_AUTH_TOKEN");
        std::env::remove_var("TWILIO_FROM_NUMBER");
        
        // Test invalid from number (missing +)
        std::env::set_var("TWILIO_ACCOUNT_SID", "ACtest");
        std::env::set_var("TWILIO_AUTH_TOKEN", "test_token");
        std::env::set_var("TWILIO_FROM_NUMBER", "15551234567");
        
        let result = TwilioConfig::from_env();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("E.164 format"));
        
        cleanup_test_env();
    }
    
    #[test]
    fn test_twilio_service_creation() {
        let config = setup_test_config();
        let service = TwilioSmsService::new(config);
        assert!(service.is_ok());
        
        let service = service.unwrap();
        assert_eq!(service.provider_name(), "Twilio");
    }
    
    #[test]
    fn test_phone_number_validation_via_trait() {
        // Clean up first
        cleanup_test_env();
        
        // We can test phone validation through the public SmsServiceTrait method
        setup_test_env();
        
        let adapter = TwilioSmsServiceAdapter::from_env().unwrap();
        
        // Test using the SmsServiceTrait method which is public
        assert!(adapter.is_valid_phone_number("+14155552671"));
        assert!(adapter.is_valid_phone_number("+919876543210"));
        assert!(adapter.is_valid_phone_number("+442071234567"));
        
        // Invalid numbers
        assert!(!adapter.is_valid_phone_number("invalid"));
        assert!(!adapter.is_valid_phone_number("123"));
        assert!(!adapter.is_valid_phone_number("+123abc"));
        
        cleanup_test_env();
    }
    
    #[test]
    fn test_message_length_note() {
        // Note: Message length validation happens in send_sms
        // which we can't test without mocking the Twilio client
        // The service enforces a 1600 character limit
        let _long_message = "a".repeat(1601);
        // This would fail in actual send_sms due to length
    }
    
    #[test]
    fn test_twilio_adapter_creation() {
        // Clean up first to ensure clean state
        cleanup_test_env();
        
        setup_test_env();
        
        let adapter = TwilioSmsServiceAdapter::from_env();
        assert!(adapter.is_ok());
        
        cleanup_test_env();
    }
    
    #[test]
    fn test_twilio_adapter_phone_validation() {
        // Clean up first
        cleanup_test_env();
        
        setup_test_env();
        
        let adapter = TwilioSmsServiceAdapter::from_env().unwrap();
        
        // Test using the SmsServiceTrait methods
        assert!(adapter.is_valid_phone_number("+14155552671"));
        assert!(adapter.is_valid_phone_number("+919876543210"));
        assert!(!adapter.is_valid_phone_number("1234567890"));
        assert!(!adapter.is_valid_phone_number("invalid"));
        
        cleanup_test_env();
    }
    
    #[tokio::test]
    async fn test_twilio_service_availability() {
        let config = setup_test_config();
        let service = TwilioSmsService::new(config).unwrap();
        
        // The service should report as available
        assert!(service.is_available().await);
    }
    
    // Integration tests would go here if we had a test Twilio account
    // For now, these are marked as ignored
    
    #[tokio::test]
    #[ignore = "Requires actual Twilio credentials"]
    async fn test_actual_sms_sending() {
        // This test would require real Twilio credentials and a test phone number
        // It's marked as ignored to prevent it from running in CI
        
        let config = TwilioConfig::from_env().expect("Twilio config from env");
        let service = TwilioSmsService::new(config).expect("Twilio service creation");
        
        let result = service.send_verification_code(
            "+14155552671", // Test phone number
            "123456"
        ).await;
        
        assert!(result.is_ok());
        println!("Message SID: {}", result.unwrap());
    }
}