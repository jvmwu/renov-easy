//! Example demonstrating OTP Redis storage with encryption and fallback
//! 
//! This example shows how to:
//! - Store encrypted OTPs in Redis
//! - Handle Redis failures with database fallback
//! - Track verification attempts
//! - Implement automatic TTL expiration

use std::sync::Arc;
use renov_infra::cache::{OtpRedisStorage, OtpStorageConfig, RedisClient};
use re_core::services::encryption::{
    encrypted_cache_trait::EncryptedCacheServiceTrait,
    otp_encryption::{AesGcmOtpEncryption, OtpEncryption, OtpEncryptionConfig},
};
use re_shared::config::cache::CacheConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    println!("OTP Redis Storage Demo");
    println!("======================\n");
    
    // 1. Setup Redis client
    let cache_config = CacheConfig {
        redis_url: std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
        pool_size: 5,
        connection_timeout_ms: 5000,
        command_timeout_ms: 1000,
        max_retries: 3,
        retry_delay_ms: 100,
    };
    
    println!("Connecting to Redis at: {}", cache_config.redis_url);
    let redis_client = RedisClient::new(cache_config).await?;
    println!("✓ Redis connection established\n");
    
    // 2. Setup encryption service
    let encryption_config = OtpEncryptionConfig {
        key_rotation: Default::default(),
        enable_db_fallback: true,
    };
    
    let encryption_service = Arc::new(AesGcmOtpEncryption::new(encryption_config.clone())?);
    println!("✓ Encryption service initialized (AES-256-GCM)\n");
    
    // 3. Setup OTP storage
    let storage_config = OtpStorageConfig {
        expiry_seconds: 300, // 5 minutes
        enable_db_fallback: true,
        max_redis_retries: 3,
        retry_delay_ms: 100,
    };
    
    let otp_storage = OtpRedisStorage::new(
        redis_client,
        encryption_config,
        None, // No database in this demo
        storage_config,
    )?;
    println!("✓ OTP storage service initialized\n");
    
    // 4. Demo: Store and verify OTP
    let phone = "+1234567890";
    let otp_code = "123456";
    
    println!("Demo: OTP Storage and Verification");
    println!("-----------------------------------");
    println!("Phone: {}", phone);
    println!("OTP Code: {}\n", otp_code);
    
    // Encrypt the OTP
    println!("Step 1: Encrypting OTP...");
    let encrypted_otp = encryption_service.encrypt_otp(otp_code, phone, 5)?;
    println!("✓ OTP encrypted");
    println!("  - Key ID: {}", encrypted_otp.key_id);
    println!("  - Expires at: {}", encrypted_otp.expires_at);
    println!("  - Ciphertext length: {} bytes\n", encrypted_otp.ciphertext.len());
    
    // Store in Redis
    println!("Step 2: Storing encrypted OTP in Redis...");
    let backend = otp_storage.store_encrypted_otp(&encrypted_otp).await?;
    println!("✓ OTP stored in: {:?}\n", backend);
    
    // Check if OTP exists
    println!("Step 3: Checking OTP existence...");
    let exists = otp_storage.encrypted_otp_exists(phone).await?;
    println!("✓ OTP exists: {}\n", exists);
    
    // Get TTL
    println!("Step 4: Getting OTP TTL...");
    if let Some(ttl) = otp_storage.get_encrypted_otp_ttl(phone).await? {
        println!("✓ OTP expires in: {} seconds\n", ttl);
    }
    
    // Retrieve and verify
    println!("Step 5: Retrieving and verifying OTP...");
    if let Some(retrieved_otp) = otp_storage.get_encrypted_otp(phone).await? {
        println!("✓ OTP retrieved from storage");
        
        // Verify with correct code
        let is_valid = encryption_service.verify_otp(&retrieved_otp, otp_code)?;
        println!("✓ Verification with correct code: {}", is_valid);
        
        // Verify with wrong code
        let is_invalid = encryption_service.verify_otp(&retrieved_otp, "000000")?;
        println!("✓ Verification with wrong code: {}\n", is_invalid);
    }
    
    // Increment attempts
    println!("Step 6: Testing attempt tracking...");
    let attempt1 = otp_storage.increment_attempt_count(phone).await?;
    println!("✓ Attempt count after 1st try: {}", attempt1);
    
    let attempt2 = otp_storage.increment_attempt_count(phone).await?;
    println!("✓ Attempt count after 2nd try: {}", attempt2);
    
    let attempt3 = otp_storage.increment_attempt_count(phone).await?;
    println!("✓ Attempt count after 3rd try: {}", attempt3);
    println!("  - Max attempts (3) reached: {}\n", attempt3 >= 3);
    
    // Test invalidation
    println!("Step 7: Testing OTP invalidation...");
    println!("Generating new OTP for same phone number...");
    let new_otp_code = "654321";
    let new_encrypted_otp = encryption_service.encrypt_otp(new_otp_code, phone, 5)?;
    
    // Storing new OTP should invalidate the old one
    let _ = otp_storage.store_encrypted_otp(&new_encrypted_otp).await?;
    println!("✓ New OTP stored (previous OTP invalidated)\n");
    
    // Clear OTP
    println!("Step 8: Clearing OTP data...");
    otp_storage.clear_encrypted_otp(phone).await?;
    println!("✓ OTP data cleared");
    
    let exists_after_clear = otp_storage.encrypted_otp_exists(phone).await?;
    println!("✓ OTP exists after clear: {}\n", exists_after_clear);
    
    // Check Redis availability
    println!("Step 9: Checking Redis availability...");
    let redis_available = otp_storage.is_redis_available().await;
    println!("✓ Redis available: {}", redis_available);
    
    let current_backend = otp_storage.get_current_backend().await;
    println!("✓ Current storage backend: {:?}\n", current_backend);
    
    println!("Demo completed successfully!");
    println!("\nSecurity features demonstrated:");
    println!("  ✓ AES-256-GCM encryption");
    println!("  ✓ Automatic TTL expiration (5 minutes)");
    println!("  ✓ Attempt tracking and limits");
    println!("  ✓ OTP invalidation on new request");
    println!("  ✓ Secure storage with metadata");
    println!("  ✓ Fallback support (database ready)");
    
    Ok(())
}