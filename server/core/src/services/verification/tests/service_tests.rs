//! Unit tests for verification service

use std::sync::Arc;

use crate::domain::entities::verification_code::CODE_LENGTH;
use crate::errors::{DomainError, ValidationError};
use crate::services::verification::{VerificationService, VerificationServiceConfig};
use crate::services::verification::CacheServiceTrait;

use super::mocks::{MockSmsService, MockCacheService};

#[tokio::test]
async fn test_send_verification_code_success() {
    let sms_service = Arc::new(MockSmsService::new(false));
    let cache_service = Arc::new(MockCacheService::new(false));
    let config = VerificationServiceConfig::default();
    
    let service = VerificationService::new(sms_service.clone(), cache_service.clone(), config);
    
    let result = service.send_verification_code("+1234567890").await;
    assert!(result.is_ok());
    
    let send_result = result.unwrap();
    assert_eq!(send_result.verification_code.phone, "+1234567890");
    assert_eq!(send_result.verification_code.code.len(), CODE_LENGTH);
    assert!(send_result.message_id.starts_with("mock-msg-"));
    
    // Verify code was sent via SMS
    let sent_code = sms_service.get_sent_code("+1234567890");
    assert_eq!(sent_code, Some(send_result.verification_code.code.clone()));
    
    // Verify code exists in cache
    assert!(cache_service.code_exists("+1234567890").await.unwrap());
}

#[tokio::test]
async fn test_send_verification_code_invalid_phone() {
    let sms_service = Arc::new(MockSmsService::new(false));
    let cache_service = Arc::new(MockCacheService::new(false));
    let config = VerificationServiceConfig::default();
    
    let service = VerificationService::new(sms_service, cache_service, config);
    
    let result = service.send_verification_code("1234567890").await; // Missing +
    assert!(result.is_err());
    
    match result.unwrap_err() {
        DomainError::Validation { message } => {
            assert!(message.contains("Invalid phone number format"));
        }
        _ => panic!("Expected validation error"),
    }
}

#[tokio::test]
async fn test_verify_code_success() {
    let sms_service = Arc::new(MockSmsService::new(false));
    let cache_service = Arc::new(MockCacheService::new(false));
    let config = VerificationServiceConfig::default();
    
    let service = VerificationService::new(sms_service, cache_service.clone(), config);
    
    // Send a code first
    let send_result = service.send_verification_code("+1234567890").await.unwrap();
    let code = send_result.verification_code.code;
    
    // Verify the correct code
    let verify_result = service.verify_code("+1234567890", &code).await.unwrap();
    assert!(verify_result.success);
    assert!(verify_result.error_message.is_none());
    assert!(verify_result.remaining_attempts.is_none());
    
    // Code should be cleared after successful verification
    assert!(!cache_service.code_exists("+1234567890").await.unwrap());
}

#[tokio::test]
async fn test_verify_code_wrong_code() {
    let sms_service = Arc::new(MockSmsService::new(false));
    let cache_service = Arc::new(MockCacheService::new(false));
    let config = VerificationServiceConfig::default();
    
    let service = VerificationService::new(sms_service, cache_service.clone(), config);
    
    // Send a code first
    service.send_verification_code("+1234567890").await.unwrap();
    
    // Verify with wrong code
    let verify_result = service.verify_code("+1234567890", "000000").await.unwrap();
    assert!(!verify_result.success);
    assert!(verify_result.error_message.is_some());
    assert_eq!(verify_result.remaining_attempts, Some(2)); // MAX_ATTEMPTS - 1
    
    // Code should still exist after failed verification
    assert!(cache_service.code_exists("+1234567890").await.unwrap());
}

#[tokio::test]
async fn test_verify_code_invalid_format() {
    let sms_service = Arc::new(MockSmsService::new(false));
    let cache_service = Arc::new(MockCacheService::new(false));
    let config = VerificationServiceConfig::default();
    
    let service = VerificationService::new(sms_service, cache_service, config);
    
    // Try to verify with invalid format codes
    let result1 = service.verify_code("+1234567890", "12345").await.unwrap(); // Too short
    assert!(!result1.success);
    assert!(result1.error_message.unwrap().contains("Invalid verification code format"));
    
    let result2 = service.verify_code("+1234567890", "12345a").await.unwrap(); // Contains letter
    assert!(!result2.success);
    assert!(result2.error_message.unwrap().contains("Invalid verification code format"));
}

#[tokio::test]
async fn test_generate_code() {
    // Test code generation multiple times
    for _ in 0..100 {
        let code = VerificationService::<MockSmsService, MockCacheService>::generate_code();
        assert_eq!(code.len(), CODE_LENGTH);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
        
        let num: u32 = code.parse().unwrap();
        assert!(num < 1_000_000);
    }
}

#[tokio::test]
async fn test_cooldown_period() {
    let sms_service = Arc::new(MockSmsService::new(false));
    let cache_service = Arc::new(MockCacheService::new(false));
    let mut config = VerificationServiceConfig::default();
    config.resend_cooldown_seconds = 60;
    
    let service = VerificationService::new(sms_service, cache_service, config);
    
    // Send first code
    let result1 = service.send_verification_code("+1234567890").await;
    assert!(result1.is_ok());
    
    // Try to send another code immediately (should fail due to cooldown)
    let result2 = service.send_verification_code("+1234567890").await;
    assert!(result2.is_err());
    
    match result2.unwrap_err() {
        DomainError::ValidationErr(ValidationError::RateLimitExceeded { .. }) => {}
        _ => panic!("Expected rate limit error"),
    }
}