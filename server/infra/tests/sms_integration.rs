//! Integration tests for SMS service functionality

use renov_infra::sms::{SmsService, MockSmsService, mask_phone_number, is_valid_phone_number, create_sms_service};
use renov_infra::config::SmsConfig;

#[tokio::test]
async fn test_complete_sms_workflow() {
    // Create service from config
    let config = SmsConfig {
        provider: "mock".to_string(),
        api_key: String::new(),
        api_secret: String::new(),
        from_number: "+1234567890".to_string(),
    };
    
    let service = create_sms_service(&config);
    
    // Test sending SMS
    let result = service.send_sms("+19875551234", "Test message").await;
    assert!(result.is_ok());
    
    // Test sending verification code
    let result = service.send_verification_code("+19875551234", "123456").await;
    assert!(result.is_ok());
    
    // Test provider name
    assert_eq!(service.provider_name(), "Mock");
    
    // Test availability
    assert!(service.is_available().await);
}

#[tokio::test]
async fn test_phone_validation_and_masking() {
    // Valid phone numbers
    assert!(is_valid_phone_number("+12345678901"));
    assert!(is_valid_phone_number("+441234567890"));
    
    // Invalid phone numbers
    assert!(!is_valid_phone_number("12345678901")); // No +
    assert!(!is_valid_phone_number("+123")); // Too short
    
    // Test masking
    assert_eq!(mask_phone_number("+12345678901"), "+*******8901");
    assert_eq!(mask_phone_number("+441234567890"), "+********7890");
}

#[tokio::test]
async fn test_mock_service_features() {
    let mut service = MockSmsService::with_options(false, false);
    
    // Send multiple messages
    for i in 1..=3 {
        let result = service.send_sms(
            "+19875551234",
            &format!("Message {}", i)
        ).await;
        assert!(result.is_ok());
        assert_eq!(service.get_message_count(), i);
    }
    
    // Reset counter
    service.reset_counter();
    assert_eq!(service.get_message_count(), 0);
    
    // Test failure simulation
    service.set_simulate_failure(true);
    let result = service.send_sms("+19875551234", "This should fail").await;
    assert!(result.is_err());
    assert!(!service.is_available().await);
}