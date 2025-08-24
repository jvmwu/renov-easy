//! Integration tests for SMS failover functionality

#[cfg(all(feature = "twilio-sms", feature = "aws-sns"))]
mod tests {
    use re_infra::sms::{
        FailoverSmsService,
        FailoverSmsServiceAdapter,
        MockSmsService,
        SmsService,
    };
    use std::time::Duration;
    use async_trait::async_trait;
    use re_infra::InfrastructureError;
    
    /// A mock SMS service that can be configured to fail
    struct ConfigurableFailureService {
        name: String,
        should_fail: std::sync::Arc<std::sync::atomic::AtomicBool>,
        failure_message: String,
    }
    
    impl ConfigurableFailureService {
        fn new(name: &str, should_fail: bool) -> Self {
            Self {
                name: name.to_string(),
                should_fail: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(should_fail)),
                failure_message: format!("{} service failed", name),
            }
        }
        
        fn set_failure(&self, should_fail: bool) {
            self.should_fail.store(should_fail, std::sync::atomic::Ordering::Relaxed);
        }
    }
    
    #[async_trait]
    impl SmsService for ConfigurableFailureService {
        async fn send_sms(&self, _phone: &str, _message: &str) -> Result<String, InfrastructureError> {
            if self.should_fail.load(std::sync::atomic::Ordering::Relaxed) {
                Err(InfrastructureError::Sms(self.failure_message.clone()))
            } else {
                Ok(format!("{}_message_id", self.name))
            }
        }
        
        fn provider_name(&self) -> &str {
            &self.name
        }
        
        async fn is_available(&self) -> bool {
            !self.should_fail.load(std::sync::atomic::Ordering::Relaxed)
        }
    }
    
    #[tokio::test]
    async fn test_failover_when_primary_fails() {
        // Create primary service that will fail
        let primary = Box::new(ConfigurableFailureService::new("Primary", true));
        
        // Create backup service that works
        let backup = Box::new(ConfigurableFailureService::new("Backup", false));
        
        // Create failover service
        let service = FailoverSmsService::new(
            primary,
            backup,
            Duration::from_secs(30),
        );
        
        // Send SMS - should use backup
        let result = service.send_sms("+1234567890", "Test message").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Backup_message_id");
    }
    
    #[tokio::test]
    async fn test_primary_recovery_after_timeout() {
        // Create services
        let primary = ConfigurableFailureService::new("Primary", true);
        let primary_ref = primary.clone();
        let backup = Box::new(ConfigurableFailureService::new("Backup", false));
        
        // Create failover service with short timeout for testing
        let service = FailoverSmsService::new(
            Box::new(primary),
            backup,
            Duration::from_millis(100),
        );
        
        // First send should fail over to backup
        let result = service.send_sms("+1234567890", "Test 1").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Backup_message_id");
        
        // Fix the primary service
        primary_ref.set_failure(false);
        
        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // Next send should try primary again and succeed
        let result = service.send_sms("+1234567890", "Test 2").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Primary_message_id");
    }
    
    #[tokio::test]
    async fn test_both_services_fail() {
        // Create both services to fail
        let primary = Box::new(ConfigurableFailureService::new("Primary", true));
        let backup = Box::new(ConfigurableFailureService::new("Backup", true));
        
        // Create failover service
        let service = FailoverSmsService::new(
            primary,
            backup,
            Duration::from_secs(30),
        );
        
        // Send SMS - should fail
        let result = service.send_sms("+1234567890", "Test message").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Both primary and backup"));
    }
    
    #[tokio::test]
    async fn test_health_check_updates_state() {
        // Create services
        let primary = ConfigurableFailureService::new("Primary", true);
        let primary_ref = primary.clone();
        let backup = Box::new(ConfigurableFailureService::new("Backup", false));
        
        // Create failover service
        let service = FailoverSmsService::new(
            Box::new(primary),
            backup,
            Duration::from_secs(30),
        );
        
        // Check availability - should detect primary is down
        let available = service.is_available().await;
        assert!(available); // Backup is available
        
        // Fix primary
        primary_ref.set_failure(false);
        
        // Check availability again - should detect primary is up
        let available = service.is_available().await;
        assert!(available);
    }
    
    #[tokio::test]
    async fn test_failover_adapter() {
        // Create services
        let primary = Box::new(MockSmsService::new());
        let backup = Box::new(MockSmsService::new());
        
        // Create failover adapter
        let adapter = FailoverSmsServiceAdapter::new(
            primary,
            backup,
            Duration::from_secs(30),
        );
        
        // Test through the trait adapter
        use re_core::services::verification::SmsServiceTrait;
        
        let result = adapter.send_verification_code("+1234567890", "123456").await;
        assert!(result.is_ok());
        
        // Test phone validation
        assert!(adapter.is_valid_phone_number("+1234567890"));
        assert!(!adapter.is_valid_phone_number("invalid"));
    }
    
    #[tokio::test]
    async fn test_failover_with_real_mock_services() {
        use re_infra::sms::create_failover_sms_service;
        
        // Set up environment for failover
        std::env::set_var("SMS_PROVIDER", "failover");
        
        // This test requires both Twilio and AWS SNS features to be enabled
        // In a real scenario, you would set up the appropriate environment variables
        // For testing, we'll just verify the function can be called
        
        let service = create_failover_sms_service().await;
        assert_eq!(service.provider_name(), "Mock"); // Falls back to mock without real configs
        
        // Clean up
        std::env::remove_var("SMS_PROVIDER");
    }
}