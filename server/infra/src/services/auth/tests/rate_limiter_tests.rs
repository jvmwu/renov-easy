//! Unit tests for rate limiter

use crate::services::auth::{RateLimitStatus, LimitInfo, RateLimitInfo};

#[test]
fn test_rate_limit_status_variants() {
    // Test Ok variant
    let ok_status = RateLimitStatus::Ok {
        remaining: 5,
        limit: 10,
        window_seconds: 3600,
    };
    
    match ok_status {
        RateLimitStatus::Ok { remaining, limit, window_seconds } => {
            assert_eq!(remaining, 5);
            assert_eq!(limit, 10);
            assert_eq!(window_seconds, 3600);
        }
        _ => panic!("Expected Ok variant"),
    }
    
    // Test Exceeded variant
    let exceeded_status = RateLimitStatus::Exceeded {
        retry_after_seconds: 120,
        limit: 10,
        window_seconds: 3600,
    };
    
    match exceeded_status {
        RateLimitStatus::Exceeded { retry_after_seconds, limit, window_seconds } => {
            assert_eq!(retry_after_seconds, 120);
            assert_eq!(limit, 10);
            assert_eq!(window_seconds, 3600);
        }
        _ => panic!("Expected Exceeded variant"),
    }
    
    // Test Locked variant
    let locked_status = RateLimitStatus::Locked {
        retry_after_seconds: 1800,
        reason: "Too many failed attempts".to_string(),
    };
    
    match locked_status {
        RateLimitStatus::Locked { retry_after_seconds, reason } => {
            assert_eq!(retry_after_seconds, 1800);
            assert_eq!(reason, "Too many failed attempts");
        }
        _ => panic!("Expected Locked variant"),
    }
}

#[test]
fn test_limit_info_structure() {
    let limit_info = LimitInfo {
        limit_type: "sms".to_string(),
        current: 2,
        limit: 3,
        window_seconds: 3600,
    };
    
    assert_eq!(limit_info.limit_type, "sms");
    assert_eq!(limit_info.current, 2);
    assert_eq!(limit_info.limit, 3);
    assert_eq!(limit_info.window_seconds, 3600);
}

#[test]
fn test_rate_limit_info_structure() {
    let limit_info = LimitInfo {
        limit_type: "sms".to_string(),
        current: 1,
        limit: 3,
        window_seconds: 3600,
    };
    
    let rate_limit_info = RateLimitInfo {
        identifier: "+1234567890".to_string(),
        identifier_type: "phone".to_string(),
        is_locked: false,
        lock_ttl_seconds: None,
        limits: vec![limit_info],
        failed_attempts: 2,
        failed_attempts_threshold: 5,
    };
    
    assert_eq!(rate_limit_info.identifier, "+1234567890");
    assert_eq!(rate_limit_info.identifier_type, "phone");
    assert!(!rate_limit_info.is_locked);
    assert!(rate_limit_info.lock_ttl_seconds.is_none());
    assert_eq!(rate_limit_info.limits.len(), 1);
    assert_eq!(rate_limit_info.failed_attempts, 2);
    assert_eq!(rate_limit_info.failed_attempts_threshold, 5);
}

#[test]
fn test_rate_limit_info_locked_state() {
    let rate_limit_info = RateLimitInfo {
        identifier: "192.168.1.1".to_string(),
        identifier_type: "ip".to_string(),
        is_locked: true,
        lock_ttl_seconds: Some(1800),
        limits: vec![],
        failed_attempts: 5,
        failed_attempts_threshold: 5,
    };
    
    assert_eq!(rate_limit_info.identifier, "192.168.1.1");
    assert_eq!(rate_limit_info.identifier_type, "ip");
    assert!(rate_limit_info.is_locked);
    assert_eq!(rate_limit_info.lock_ttl_seconds, Some(1800));
    assert_eq!(rate_limit_info.failed_attempts, 5);
}

#[test]
fn test_hash_phone_consistency() {
    use sha2::{Sha256, Digest};
    
    fn hash_phone(phone: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(phone.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    let phone = "+1234567890";
    let hash1 = hash_phone(phone);
    let hash2 = hash_phone(phone);
    
    // Same input should produce same hash
    assert_eq!(hash1, hash2);
    
    // Hash should be 64 characters (SHA256 hex output)
    assert_eq!(hash1.len(), 64);
    
    // Hash should be hexadecimal
    assert!(hash1.chars().all(|c| c.is_ascii_hexdigit()));
    
    // Different phones should produce different hashes
    let different_phone = "+0987654321";
    let different_hash = hash_phone(different_phone);
    assert_ne!(hash1, different_hash);
}

