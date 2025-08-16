//! Example demonstrating the VerificationCache service usage
//! 
//! Run with: cargo run --example verification_cache_demo

use infrastructure::cache::{RedisClient, VerificationCache};
use infrastructure::config::CacheConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create Redis client with configuration
    let config = CacheConfig {
        url: std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        pool_size: 5,
        default_ttl: 3600,
    };

    println!("Connecting to Redis...");
    let redis_client = match RedisClient::new(config).await {
        Ok(client) => {
            println!("✓ Connected to Redis successfully");
            client
        }
        Err(e) => {
            println!("✗ Failed to connect to Redis: {}", e);
            println!("  Make sure Redis is running on localhost:6379");
            return Ok(());
        }
    };

    // Create verification cache service
    let verification_service = VerificationCache::new(redis_client);

    // Demo phone number and code
    let phone = "1234567890";
    let verification_code = "123456";

    println!("\n=== Verification Cache Demo ===\n");

    // 1. Store a verification code
    println!("1. Storing verification code for phone: {}", phone);
    verification_service
        .store_code(phone, verification_code)
        .await?;
    println!("   ✓ Code stored successfully (expires in 5 minutes)");

    // 2. Check if code exists
    let exists = verification_service.code_exists(phone).await?;
    println!("\n2. Checking if code exists: {}", exists);

    // 3. Get TTL
    if let Some(ttl) = verification_service.get_code_ttl(phone).await? {
        println!("   Code expires in {} seconds", ttl);
    }

    // 4. Get remaining attempts
    let remaining = verification_service.get_remaining_attempts(phone).await?;
    println!("\n3. Remaining verification attempts: {}/3", remaining);

    // 5. Try with wrong code
    println!("\n4. Attempting verification with wrong code...");
    let wrong_code = "000000";
    let result = verification_service.verify_code(phone, wrong_code).await?;
    println!("   Wrong code verification result: {}", result);
    
    let remaining = verification_service.get_remaining_attempts(phone).await?;
    println!("   Remaining attempts: {}/3", remaining);

    // 6. Try with correct code
    println!("\n5. Attempting verification with correct code...");
    let result = verification_service.verify_code(phone, verification_code).await?;
    println!("   Correct code verification result: {}", result);

    if result {
        println!("   ✓ Verification successful!");
        println!("   Code and attempts cleared automatically");
    }

    // 7. Verify code is deleted after successful verification
    let exists_after = verification_service.code_exists(phone).await?;
    println!("\n6. Code exists after successful verification: {}", exists_after);

    println!("\n=== Demo Complete ===");

    Ok(())
}