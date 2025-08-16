//! Example demonstrating the VerificationService integration
//!
//! This example shows how to use the VerificationService from the core module
//! with the infrastructure implementations of SMS and cache services.
//!
//! Run with: cargo run --example verification_service_demo

use std::sync::Arc;
use async_trait::async_trait;

use renov_core::services::{
    VerificationService, VerificationServiceConfig,
    SmsServiceTrait, CacheServiceTrait,
};
use renov_infrastructure::{
    cache::{verification_cache::VerificationCache, RedisClient},
    config::{CacheConfig, InfrastructureConfig},
    sms::{sms_service::SmsService, mock_sms::MockSmsService},
    InfrastructureError,
};

/// Adapter to bridge infrastructure SmsService to core SmsServiceTrait
struct SmsServiceAdapter {
    inner: Arc<dyn SmsService>,
}

#[async_trait]
impl SmsServiceTrait for SmsServiceAdapter {
    async fn send_verification_code(&self, phone: &str, code: &str) -> Result<String, String> {
        self.inner
            .send_verification_code(phone, code)
            .await
            .map_err(|e| e.to_string())
    }

    fn is_valid_phone_number(&self, phone: &str) -> bool {
        renov_infrastructure::sms::sms_service::is_valid_phone_number(phone)
    }
}

/// Adapter to bridge infrastructure VerificationCache to core CacheServiceTrait
struct CacheServiceAdapter {
    inner: VerificationCache,
}

#[async_trait]
impl CacheServiceTrait for CacheServiceAdapter {
    async fn store_code(&self, phone: &str, code: &str) -> Result<(), String> {
        self.inner.store_code(phone, code).await.map_err(|e| e.to_string())
    }

    async fn verify_code(&self, phone: &str, code: &str) -> Result<bool, String> {
        self.inner.verify_code(phone, code).await.map_err(|e| e.to_string())
    }

    async fn get_remaining_attempts(&self, phone: &str) -> Result<i64, String> {
        self.inner.get_remaining_attempts(phone).await.map_err(|e| e.to_string())
    }

    async fn code_exists(&self, phone: &str) -> Result<bool, String> {
        self.inner.code_exists(phone).await.map_err(|e| e.to_string())
    }

    async fn get_code_ttl(&self, phone: &str) -> Result<Option<i64>, String> {
        self.inner.get_code_ttl(phone).await.map_err(|e| e.to_string())
    }

    async fn clear_verification(&self, phone: &str) -> Result<(), String> {
        self.inner.clear_verification(phone).await.map_err(|e| e.to_string())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== Verification Service Demo ===\n");

    // Load configuration
    let config = InfrastructureConfig::from_env();
    
    // Initialize Redis client for cache
    println!("Connecting to Redis...");
    let redis_client = RedisClient::new(config.cache.clone()).await?;
    let verification_cache = VerificationCache::new(redis_client);
    println!("✓ Redis connected\n");

    // Initialize SMS service (using mock for demo)
    println!("Initializing SMS service (mock mode)...");
    let sms_service: Arc<dyn SmsService> = Arc::new(MockSmsService::new());
    println!("✓ SMS service ready\n");

    // Create adapters
    let sms_adapter = Arc::new(SmsServiceAdapter {
        inner: sms_service,
    });
    let cache_adapter = Arc::new(CacheServiceAdapter {
        inner: verification_cache,
    });

    // Create verification service
    let verification_config = VerificationServiceConfig {
        code_expiration_minutes: 5,
        max_attempts: 3,
        use_mock_sms: true,
        resend_cooldown_seconds: 60,
    };
    
    let verification_service = VerificationService::new(
        sms_adapter,
        cache_adapter,
        verification_config,
    );

    // Demo: Send verification code
    let phone = "+61412345678";
    println!("Sending verification code to {}...", phone);
    
    match verification_service.send_verification_code(phone).await {
        Ok(result) => {
            println!("✓ Verification code sent successfully!");
            println!("  - Code: {} (visible in mock mode)", result.verification_code.code);
            println!("  - Message ID: {}", result.message_id);
            println!("  - Expires at: {}", result.verification_code.expires_at);
            println!("  - Next resend at: {}", result.next_resend_at);
            
            // Demo: Verify the code
            println!("\n--- Attempting to verify the code ---");
            let verify_result = verification_service
                .verify_code(phone, &result.verification_code.code)
                .await?;
            
            if verify_result.success {
                println!("✓ Code verified successfully!");
            } else {
                println!("✗ Verification failed: {:?}", verify_result.error_message);
            }
            
            // Demo: Try to verify again (should fail as code is already used)
            println!("\n--- Attempting to verify the same code again ---");
            let verify_result2 = verification_service
                .verify_code(phone, &result.verification_code.code)
                .await?;
            
            if !verify_result2.success {
                println!("✓ Correctly rejected already-used code");
                println!("  Error: {:?}", verify_result2.error_message);
            }
        }
        Err(e) => {
            println!("✗ Failed to send verification code: {}", e);
        }
    }

    // Demo: Test wrong code
    println!("\n--- Testing wrong code with new verification ---");
    
    // Clear previous verification
    verification_service.clear_verification(phone).await?;
    
    // Send new code
    let result = verification_service.send_verification_code(phone).await?;
    println!("New code sent: {}", result.verification_code.code);
    
    // Try wrong code
    let wrong_code = "000000";
    let verify_result = verification_service.verify_code(phone, wrong_code).await?;
    
    if !verify_result.success {
        println!("✓ Correctly rejected wrong code");
        println!("  Remaining attempts: {:?}", verify_result.remaining_attempts);
        println!("  Error: {:?}", verify_result.error_message);
    }

    // Demo: Test rate limiting
    println!("\n--- Testing rate limiting ---");
    match verification_service.send_verification_code(phone).await {
        Err(e) => {
            println!("✓ Rate limiting working: {}", e);
        }
        Ok(_) => {
            println!("✗ Rate limiting not working (unexpected success)");
        }
    }

    println!("\n=== Demo completed successfully ===");
    Ok(())
}