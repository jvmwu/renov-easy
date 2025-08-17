//! Unit tests for verification code entity

use std::thread;
use std::time::Duration as StdDuration;
use chrono::Duration;
use crate::domain::entities::verification_code::{
    VerificationCode, MAX_ATTEMPTS, CODE_LENGTH, DEFAULT_EXPIRATION_MINUTES
};

#[test]
fn test_new_verification_code() {
    let phone = "+61412345678".to_string();
    let code = VerificationCode::new(phone.clone());
    
    assert_eq!(code.phone, phone);
    assert_eq!(code.code.len(), CODE_LENGTH);
    assert_eq!(code.attempts, 0);
    assert!(!code.is_used);
    assert!(!code.is_expired());
    assert!(code.is_valid());
}

#[test]
fn test_generate_code_format() {
    // Test multiple times to ensure consistency
    for _ in 0..100 {
        let code = VerificationCode::generate_code();
        assert_eq!(code.len(), CODE_LENGTH);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
        
        // Verify it's a valid number
        let num: u32 = code.parse().expect("Generated code should be a valid number");
        assert!(num < 1_000_000);
    }
}

#[test]
fn test_code_uniqueness() {
    // Generate multiple codes and check they're not all the same
    let codes: Vec<String> = (0..100)
        .map(|_| VerificationCode::generate_code())
        .collect();
    
    // There should be at least some unique codes (extremely unlikely to get all same)
    let unique_count = codes.iter().collect::<std::collections::HashSet<_>>().len();
    assert!(unique_count > 1);
}

#[test]
fn test_verification_success() {
    let mut code = VerificationCode::new("+61412345678".to_string());
    let verification_code = code.code.clone();
    
    let result = code.verify(&verification_code);
    assert!(result.is_ok());
    assert!(code.is_used);
    assert_eq!(code.attempts, 1);
}

#[test]
fn test_verification_failure() {
    let mut code = VerificationCode::new("+61412345678".to_string());
    
    let result = code.verify("000000");
    assert!(result.is_err());
    assert!(!code.is_used);
    assert_eq!(code.attempts, 1);
    assert_eq!(code.remaining_attempts(), 2);
}

#[test]
fn test_max_attempts() {
    let mut code = VerificationCode::new("+61412345678".to_string());
    let correct_code = code.code.clone();
    
    // Make MAX_ATTEMPTS with wrong code
    for i in 1..=MAX_ATTEMPTS {
        let result = code.verify("000000");
        assert!(result.is_err());
        assert_eq!(code.attempts, i);
    }
    
    // Next attempt should fail due to max attempts
    let result = code.verify(&correct_code);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Maximum verification attempts exceeded"));
}

#[test]
fn test_already_used_code() {
    let mut code = VerificationCode::new("+61412345678".to_string());
    let verification_code = code.code.clone();
    
    // First verification should succeed
    assert!(code.verify(&verification_code).is_ok());
    
    // Second verification should fail because code is already used
    let result = code.verify(&verification_code);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already been used"));
}

#[test]
fn test_custom_expiration() {
    let phone = "+61412345678".to_string();
    let expiration_minutes = 10;
    let code = VerificationCode::new_with_expiration(phone, expiration_minutes);
    
    let expected_expiration = code.created_at + Duration::minutes(expiration_minutes);
    assert_eq!(code.expires_at, expected_expiration);
}

#[test]
fn test_is_expired() {
    // Create a code that expires immediately (0 minutes)
    let mut code = VerificationCode::new_with_expiration("+61412345678".to_string(), 0);
    let verification_code = code.code.clone();
    
    // Sleep for a short time to ensure expiration
    thread::sleep(StdDuration::from_millis(10));
    
    assert!(code.is_expired());
    assert!(!code.is_valid());
    
    // Verification should fail due to expiration
    let result = code.verify(&verification_code);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("expired"));
}

#[test]
fn test_reset_code() {
    let mut code = VerificationCode::new("+61412345678".to_string());
    let original_code = code.code.clone();
    
    // Make some attempts
    code.verify("000000").ok();
    code.verify("111111").ok();
    assert_eq!(code.attempts, 2);
    
    // Reset the code
    code.reset();
    
    // Check that everything is reset
    assert_ne!(code.code, original_code); // Should have a new code
    assert_eq!(code.attempts, 0);
    assert!(!code.is_used);
    assert!(code.is_valid());
}

#[test]
fn test_remaining_attempts() {
    let mut code = VerificationCode::new("+61412345678".to_string());
    
    assert_eq!(code.remaining_attempts(), MAX_ATTEMPTS);
    
    code.verify("000000").ok();
    assert_eq!(code.remaining_attempts(), MAX_ATTEMPTS - 1);
    
    code.verify("111111").ok();
    assert_eq!(code.remaining_attempts(), MAX_ATTEMPTS - 2);
    
    code.verify("222222").ok();
    assert_eq!(code.remaining_attempts(), 0);
}

#[test]
fn test_time_until_expiration() {
    let code = VerificationCode::new("+61412345678".to_string());
    
    let time_remaining = code.time_until_expiration();
    assert!(time_remaining <= Duration::minutes(DEFAULT_EXPIRATION_MINUTES));
    assert!(time_remaining > Duration::minutes(DEFAULT_EXPIRATION_MINUTES - 1));
}

#[test]
fn test_serialization() {
    let code = VerificationCode::new("+61412345678".to_string());
    
    // Serialize to JSON
    let json = serde_json::to_string(&code).unwrap();
    
    // Deserialize back
    let deserialized: VerificationCode = serde_json::from_str(&json).unwrap();
    
    assert_eq!(code, deserialized);
}

#[test]
fn test_mark_as_used() {
    let mut code = VerificationCode::new("+61412345678".to_string());
    assert!(!code.is_used);
    
    code.mark_as_used();
    assert!(code.is_used);
    assert!(!code.is_valid());
}