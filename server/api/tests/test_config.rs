//! Example program to test configuration loading

use api::config::Config;
use std::env;

fn main() {
    // Set up some test environment variables
    env::set_var("ENVIRONMENT", "development");
    
    println!("Testing configuration loading...\n");
    
    // Test development configuration
    match Config::from_env() {
        Ok(config) => {
            println!("✅ Configuration loaded successfully!");
            println!("Environment: {:?}", config.environment);
            println!("Database URL: {}", config.database_url());
            println!("Redis URL: {}", config.redis_url());
            println!("Server: {}:{}", config.server_host(), config.server_port());
            println!("JWT Secret (first 10 chars): {}...", &config.jwt_secret()[..10.min(config.jwt_secret().len())]);
            println!("SMS Provider: {}", config.sms_provider());
            println!("SMS API Key: {:?}", config.sms_api_key());
            println!("Rate Limiting Enabled: {}", config.rate_limit.enabled);
            println!("\nValidation Result: {:?}", config.validate());
        }
        Err(e) => {
            println!("❌ Failed to load configuration: {}", e);
        }
    }
    
    println!("\nTesting production configuration requirements...");
    env::set_var("ENVIRONMENT", "production");
    
    match Config::from_env() {
        Ok(_) => {
            println!("❌ Production config should fail without required env vars");
        }
        Err(e) => {
            println!("✅ Production config correctly requires env vars: {}", e);
        }
    }
    
    // Test with production env vars set
    println!("\nTesting production with required env vars...");
    env::set_var("DATABASE_URL", "mysql://prod-db:3306/renoveasy");
    env::set_var("REDIS_URL", "redis://prod-redis:6379");
    env::set_var("JWT_SECRET", "super-secret-production-key");
    env::set_var("SMS_PROVIDER", "twilio");
    env::set_var("SMS_API_KEY", "test-api-key");
    env::set_var("SMS_API_SECRET", "test-api-secret");
    env::set_var("SMS_SENDER_ID", "+1234567890");
    
    match Config::from_env() {
        Ok(config) => {
            println!("✅ Production configuration loaded!");
            println!("Environment: {:?}", config.environment);
            println!("Database URL: {}", config.database_url());
            println!("SMS Provider: {}", config.sms_provider());
            println!("Validation Result: {:?}", config.validate());
        }
        Err(e) => {
            println!("❌ Failed to load production config: {}", e);
        }
    }
}