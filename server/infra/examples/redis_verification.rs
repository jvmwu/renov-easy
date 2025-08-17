//! Example: Using Redis cache for SMS verification codes
//! 
//! This example demonstrates how to use the Redis client for storing
//! and validating SMS verification codes with expiry and rate limiting.
//! 
//! Run with: cargo run --example redis_verification -p infra

use infra::cache::{CacheConfig, RedisClient};
use infra::InfrastructureError;
use rand::Rng;
use std::time::Duration;

/// Verification code service using Redis cache
struct VerificationService {
    redis_client: RedisClient,
    code_expiry_seconds: u64,
    max_attempts: i64,
}

impl VerificationService {
    /// Create a new verification service
    async fn new(redis_url: String) -> Result<Self, InfrastructureError> {
        let config = CacheConfig {
            url: redis_url,
            pool_size: 10,
            default_ttl: 3600,
        };

        let redis_client = RedisClient::new(config).await?;

        Ok(Self {
            redis_client,
            code_expiry_seconds: 300, // 5 minutes
            max_attempts: 5,
        })
    }

    /// Generate a 6-digit verification code
    fn generate_code() -> String {
        let mut rng = rand::thread_rng();
        let code: u32 = rng.gen_range(100000..1000000);
        code.to_string()
    }

    /// Send verification code (stores in cache)
    async fn send_code(&self, phone: &str) -> Result<String, InfrastructureError> {
        // Check rate limit (max 3 codes per hour)
        let rate_limit_key = format!("rate_limit:sms:{}", phone);
        let request_count = self
            .redis_client
            .increment(&rate_limit_key, Some(3600))
            .await?;

        if request_count > 3 {
            return Err(InfrastructureError::Sms(
                "Too many requests. Please try again later.".to_string(),
            ));
        }

        // Generate and store code
        let code = Self::generate_code();
        let code_key = format!("verification:{}", phone);
        
        self.redis_client
            .set_with_expiry(&code_key, &code, self.code_expiry_seconds)
            .await?;

        // Reset attempt counter
        let attempt_key = format!("verification:attempts:{}", phone);
        self.redis_client.delete(&attempt_key).await?;

        println!("Verification code {} sent to {}", code, phone);
        Ok(code)
    }

    /// Verify a code
    async fn verify_code(&self, phone: &str, code: &str) -> Result<bool, InfrastructureError> {
        // Check attempt counter
        let attempt_key = format!("verification:attempts:{}", phone);
        let attempts = self
            .redis_client
            .increment(&attempt_key, Some(self.code_expiry_seconds))
            .await?;

        if attempts > self.max_attempts {
            return Err(InfrastructureError::Sms(
                "Maximum verification attempts exceeded".to_string(),
            ));
        }

        // Get stored code
        let code_key = format!("verification:{}", phone);
        let stored_code = self.redis_client.get(&code_key).await?;

        match stored_code {
            Some(ref stored) if stored == code => {
                // Successful verification - delete the code
                self.redis_client.delete(&code_key).await?;
                self.redis_client.delete(&attempt_key).await?;
                Ok(true)
            }
            Some(_) => {
                // Wrong code
                println!(
                    "Invalid code for {}. Attempt {}/{}",
                    phone, attempts, self.max_attempts
                );
                Ok(false)
            }
            None => {
                // Code expired or doesn't exist
                Err(InfrastructureError::Sms(
                    "Verification code expired or not found".to_string(),
                ))
            }
        }
    }

    /// Get remaining time for a verification code
    async fn get_code_ttl(&self, phone: &str) -> Result<Option<i64>, InfrastructureError> {
        let code_key = format!("verification:{}", phone);
        self.redis_client.ttl(&code_key).await
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create verification service
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    
    let service = VerificationService::new(redis_url).await?;

    // Example phone number
    let phone = "13800138000";

    // Send verification code
    let code = service.send_code(phone).await?;
    println!("Code sent successfully");

    // Check TTL
    if let Some(ttl) = service.get_code_ttl(phone).await? {
        println!("Code expires in {} seconds", ttl);
    }

    // Simulate user entering wrong code
    println!("\nTrying wrong code...");
    let wrong_result = service.verify_code(phone, "000000").await?;
    println!("Wrong code verification result: {}", wrong_result);

    // Verify with correct code
    println!("\nTrying correct code...");
    let correct_result = service.verify_code(phone, &code).await?;
    println!("Correct code verification result: {}", correct_result);

    // Try to verify again (should fail as code is deleted after successful verification)
    println!("\nTrying to reuse code...");
    match service.verify_code(phone, &code).await {
        Ok(result) => println!("Reuse attempt result: {}", result),
        Err(e) => println!("Reuse attempt failed as expected: {}", e),
    }

    // Simulate rate limiting
    println!("\nTesting rate limiting...");
    for i in 1..=5 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        match service.send_code(&format!("1390000000{}", i)).await {
            Ok(_) => println!("Request {} succeeded", i),
            Err(e) => println!("Request {} rate limited: {}", i, e),
        }
    }

    println!("\nExample completed successfully!");
    Ok(())
}