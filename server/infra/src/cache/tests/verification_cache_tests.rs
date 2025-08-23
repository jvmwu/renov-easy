//! Unit tests for verification cache

use crate::cache::verification_cache::VerificationCache;
use crate::cache::redis_client::RedisClient;
use re_shared::config::cache::CacheConfig;

#[test]
fn test_format_keys() {
    let phone = "1234567890";
    
    assert_eq!(
        VerificationCache::format_code_key(phone),
        "verification:code:1234567890"
    );
    
    assert_eq!(
        VerificationCache::format_attempts_key(phone),
        "verification:attempts:1234567890"
    );
}

#[test]
fn test_hash_code() {
    let code1 = "123456";
    let code2 = "654321";
    let code1_duplicate = "123456";
    
    let hash1 = VerificationCache::hash_code(code1);
    let hash2 = VerificationCache::hash_code(code2);
    let hash1_dup = VerificationCache::hash_code(code1_duplicate);
    
    // Same code should produce same hash
    assert_eq!(hash1, hash1_dup);
    
    // Different codes should produce different hashes
    assert_ne!(hash1, hash2);
    
    // Hash should be hex string (64 chars for SHA-256)
    assert_eq!(hash1.len(), 64);
    assert!(hash1.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_mask_phone() {
    assert_eq!(VerificationCache::mask_phone("1234567890"), "***7890");
    assert_eq!(VerificationCache::mask_phone("567890"), "***7890");
    assert_eq!(VerificationCache::mask_phone("1234"), "****");
    assert_eq!(VerificationCache::mask_phone("123"), "****");
    assert_eq!(VerificationCache::mask_phone(""), "****");
}

#[tokio::test]
#[ignore] // Requires actual Redis server
async fn test_store_and_verify_code() {
    let config = CacheConfig::new(
        std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string())
    );

    let redis_client = RedisClient::new(config).await.unwrap();
    let service = VerificationCache::new(redis_client);
    
    let phone = "test_1234567890";
    let code = "123456";
    
    // Clean up from previous tests
    service.clear_verification(phone).await.unwrap();
    
    // Store code
    service.store_code(phone, code).await.unwrap();
    
    // Verify correct code
    let valid = service.verify_code(phone, code).await.unwrap();
    assert!(valid);
    
    // Code should be deleted after successful verification
    let exists = service.code_exists(phone).await.unwrap();
    assert!(!exists);
}

#[tokio::test]
#[ignore] // Requires actual Redis server
async fn test_verify_incorrect_code() {
    let config = CacheConfig::new(
        std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string())
    );

    let redis_client = RedisClient::new(config).await.unwrap();
    let service = VerificationCache::new(redis_client);
    
    let phone = "test_9876543210";
    let correct_code = "123456";
    let wrong_code = "654321";
    
    // Clean up from previous tests
    service.clear_verification(phone).await.unwrap();
    
    // Store code
    service.store_code(phone, correct_code).await.unwrap();
    
    // Verify incorrect code
    let valid = service.verify_code(phone, wrong_code).await.unwrap();
    assert!(!valid);
    
    // Code should still exist after failed attempt
    let exists = service.code_exists(phone).await.unwrap();
    assert!(exists);
    
    // Should have 2 remaining attempts
    let remaining = service.get_remaining_attempts(phone).await.unwrap();
    assert_eq!(remaining, 2);
}

#[tokio::test]
#[ignore] // Requires actual Redis server
async fn test_max_attempts() {
    let config = CacheConfig::new(
        std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string())
    );

    let redis_client = RedisClient::new(config).await.unwrap();
    let service = VerificationCache::new(redis_client);
    
    let phone = "test_5555555555";
    let correct_code = "123456";
    let wrong_code = "000000";
    
    // Clean up from previous tests
    service.clear_verification(phone).await.unwrap();
    
    // Store code
    service.store_code(phone, correct_code).await.unwrap();
    
    // Make 3 failed attempts
    for i in 1..=3 {
        let valid = service.verify_code(phone, wrong_code).await.unwrap();
        assert!(!valid, "Attempt {} should fail", i);
    }
    
    // 4th attempt should fail even with correct code (max attempts exceeded)
    let valid = service.verify_code(phone, correct_code).await.unwrap();
    assert!(!valid, "Should fail due to max attempts");
    
    // Remaining attempts should be 0
    let remaining = service.get_remaining_attempts(phone).await.unwrap();
    assert_eq!(remaining, 0);
}

#[tokio::test]
#[ignore] // Requires actual Redis server  
async fn test_get_code_ttl() {
    let config = CacheConfig::new(
        std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string())
    );

    let redis_client = RedisClient::new(config).await.unwrap();
    let service = VerificationCache::new(redis_client);
    
    let phone = "test_ttl_check";
    let code = "123456";
    
    // Clean up from previous tests
    service.clear_verification(phone).await.unwrap();
    
    // Store code
    service.store_code(phone, code).await.unwrap();
    
    // Check TTL
    let ttl = service.get_code_ttl(phone).await.unwrap();
    assert!(ttl.is_some());
    let ttl_value = ttl.unwrap();
    // Use CODE_EXPIRY_SECONDS constant from the module
    assert!(ttl_value > 0 && ttl_value <= 300);
    
    // Clean up
    service.clear_verification(phone).await.unwrap();
}