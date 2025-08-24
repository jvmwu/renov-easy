//! Unit tests for verification service

use std::sync::Arc;

use crate::domain::entities::verification_code::CODE_LENGTH;
use crate::errors::{DomainError, ValidationError};
use crate::services::verification::{VerificationService, VerificationServiceConfig};
use crate::services::verification::CacheServiceTrait;
use crate::services::verification::service::OtpMetadata;
use chrono::{Utc, Duration};

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
async fn test_generate_secure_code() {
    // Test secure code generation multiple times
    for _ in 0..100 {
        let code = VerificationService::<MockSmsService, MockCacheService>::generate_secure_code();
        assert_eq!(code.len(), CODE_LENGTH);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
        
        let num: u32 = code.parse().unwrap();
        assert!(num < 1_000_000);
    }
}

#[tokio::test]
async fn test_generate_secure_code_uniqueness() {
    // Test that codes are reasonably unique (not deterministic)
    let mut codes = std::collections::HashSet::new();
    for _ in 0..100 {
        let code = VerificationService::<MockSmsService, MockCacheService>::generate_secure_code();
        codes.insert(code);
    }
    // We should have at least 90 unique codes out of 100 (allowing for some collisions)
    assert!(codes.len() >= 90, "Generated codes lack sufficient randomness");
}

#[tokio::test]
async fn test_constant_time_comparison() {
    // Test equal strings
    assert!(VerificationService::<MockSmsService, MockCacheService>::verify_code_constant_time(
        "123456", 
        "123456"
    ));
    
    // Test different strings of same length
    assert!(!VerificationService::<MockSmsService, MockCacheService>::verify_code_constant_time(
        "123456", 
        "123457"
    ));
    
    // Test different lengths (should return false immediately)
    assert!(!VerificationService::<MockSmsService, MockCacheService>::verify_code_constant_time(
        "12345", 
        "123456"
    ));
    
    // Test empty strings
    assert!(VerificationService::<MockSmsService, MockCacheService>::verify_code_constant_time(
        "", 
        ""
    ));
    
    // Test strings that differ at the beginning
    assert!(!VerificationService::<MockSmsService, MockCacheService>::verify_code_constant_time(
        "923456", 
        "123456"
    ));
    
    // Test strings that differ at the end
    assert!(!VerificationService::<MockSmsService, MockCacheService>::verify_code_constant_time(
        "123459", 
        "123456"
    ));
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

#[tokio::test]
async fn test_invalidate_previous_codes() {
    let sms_service = Arc::new(MockSmsService::new(false));
    let cache_service = Arc::new(MockCacheService::new(false));
    // Create custom config with 5 minute expiration and 0 cooldown for this test
    let config = VerificationServiceConfig {
        code_expiration_minutes: 5,
        resend_cooldown_seconds: 0, // No cooldown for testing invalidation
        max_attempts: 3,
        use_mock_sms: false,
    };
    
    let service = VerificationService::new(sms_service, cache_service.clone(), config);
    
    // Send first code
    let result1 = service.send_verification_code("+1234567890").await;
    assert!(result1.is_ok());
    let code1 = result1.unwrap().verification_code.code;
    
    // Send second code immediately (should invalidate first due to no cooldown)
    let result2 = service.send_verification_code("+1234567890").await;
    assert!(result2.is_ok());
    let code2 = result2.unwrap().verification_code.code;
    
    // Codes should be different (using CSPRNG)
    assert_ne!(code1, code2);
    
    // Only the second code should work
    let verify_result = service.verify_code("+1234567890", &code2).await;
    assert!(verify_result.is_ok());
    assert!(verify_result.unwrap().success);
}

#[tokio::test]
async fn test_mark_code_as_used() {
    let sms_service = Arc::new(MockSmsService::new(false));
    let cache_service = Arc::new(MockCacheService::new(false));
    let config = VerificationServiceConfig::default();
    
    let service = VerificationService::new(sms_service, cache_service.clone(), config);
    
    // Send code
    let send_result = service.send_verification_code("+1234567890").await.unwrap();
    let code = send_result.verification_code.code;
    
    // Verify code (should mark as used)
    let verify_result = service.verify_code("+1234567890", &code).await.unwrap();
    assert!(verify_result.success);
    
    // Code should be cleared after successful verification
    assert!(!cache_service.code_exists("+1234567890").await.unwrap());
}

#[tokio::test]
async fn test_max_attempts_exceeded() {
    let sms_service = Arc::new(MockSmsService::new(false));
    let cache_service = Arc::new(MockCacheService::new(false));
    let config = VerificationServiceConfig::default();
    
    let service = VerificationService::new(sms_service, cache_service.clone(), config);
    
    // Send code
    service.send_verification_code("+1234567890").await.unwrap();
    
    // Try wrong code 3 times
    for i in 1..=3 {
        let verify_result = service.verify_code("+1234567890", "000000").await.unwrap();
        assert!(!verify_result.success);
        
        if i < 3 {
            assert_eq!(verify_result.remaining_attempts, Some(3 - i));
        } else {
            assert_eq!(verify_result.remaining_attempts, Some(0));
            assert!(verify_result.error_message.unwrap().contains("Maximum verification attempts exceeded"));
        }
    }
    
    // 4th attempt should still fail
    let verify_result = service.verify_code("+1234567890", "000000").await.unwrap();
    assert!(!verify_result.success);
    assert_eq!(verify_result.remaining_attempts, Some(0));
}

#[tokio::test]
async fn test_concurrent_code_generation() {
    use std::sync::Arc;
    
    // Test that concurrent code generation is thread-safe
    let sms_service = Arc::new(MockSmsService::new(false));
    let cache_service = Arc::new(MockCacheService::new(false));
    let config = VerificationServiceConfig::default();
    let service = Arc::new(VerificationService::new(sms_service, cache_service, config));
    
    let mut handles = vec![];
    
    for i in 0..10 {
        let service_clone = Arc::clone(&service);
        let handle = tokio::spawn(async move {
            let phone = format!("+123456789{}", i);
            service_clone.send_verification_code(&phone).await
        });
        handles.push(handle);
    }
    
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
}

#[test]
fn test_otp_metadata_structure() {
    let metadata = OtpMetadata {
        code: "123456".to_string(),
        created_at: Utc::now(),
        expires_at: Utc::now() + Duration::minutes(5),
        attempts: 0,
        max_attempts: 3,
        is_used: false,
        phone: "+1234567890".to_string(),
        session_id: "test-session-id".to_string(),
    };
    
    assert_eq!(metadata.code, "123456");
    assert_eq!(metadata.max_attempts, 3);
    assert!(!metadata.is_used);
    assert_eq!(metadata.phone, "+1234567890");
    assert_eq!(metadata.session_id, "test-session-id");
}