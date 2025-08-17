//! Unit tests for mock SMS service

use crate::sms::{SmsService, MockSmsService};
use crate::InfrastructureError;

#[tokio::test]
async fn test_mock_sms_send_success() {
    let service = MockSmsService::with_options(false, false);
    let result = service.send_sms("+1234567890", "Test message").await;
    
    assert!(result.is_ok());
    let message_id = result.unwrap();
    assert!(message_id.starts_with("mock_"));
    assert_eq!(service.get_message_count(), 1);
}

#[tokio::test]
async fn test_mock_sms_invalid_phone() {
    let service = MockSmsService::new();
    let result = service.send_sms("1234567890", "Test message").await;
    
    assert!(result.is_err());
    if let Err(InfrastructureError::Sms(msg)) = result {
        assert!(msg.contains("Invalid phone number"));
    } else {
        panic!("Expected Sms error");
    }
}

#[tokio::test]
async fn test_mock_sms_simulate_failure() {
    let mut service = MockSmsService::new();
    service.set_simulate_failure(true);
    
    let result = service.send_sms("+1234567890", "Test message").await;
    assert!(result.is_err());
    assert!(!service.is_available().await);
}

#[tokio::test]
async fn test_mock_sms_verification_code() {
    let service = MockSmsService::with_options(false, false);
    let result = service.send_verification_code("+1234567890", "123456").await;
    
    assert!(result.is_ok());
    assert_eq!(service.get_message_count(), 1);
}

#[tokio::test]
async fn test_mock_sms_counter() {
    let service = MockSmsService::new();
    
    for i in 1..=3 {
        let _ = service.send_sms("+1234567890", &format!("Message {}", i)).await;
        assert_eq!(service.get_message_count(), i);
    }
    
    service.reset_counter();
    assert_eq!(service.get_message_count(), 0);
}

#[test]
fn test_provider_name() {
    let service = MockSmsService::new();
    assert_eq!(service.provider_name(), "Mock");
}