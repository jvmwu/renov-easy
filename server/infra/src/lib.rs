//! # Infrastructure Layer
//! 
//! This crate implements the infrastructure layer for the RenovEasy application,
//! following Clean Architecture principles. It provides concrete implementations
//! for external services, database access, caching, and SMS services.
//!
//! ## Architecture
//! 
//! The infrastructure layer contains:
//! - **Database**: MySQL implementations using SQLx
//! - **Cache**: Redis client for caching and rate limiting
//! - **SMS**: SMS service integrations (Twilio, AWS SNS)
//! - **External APIs**: HTTP client implementations
//!
//! ## Features
//!
//! - `mysql`: Enable MySQL database support (default)
//! - `redis-cache`: Enable Redis caching support (default) 
//! - `twilio-sms`: Enable Twilio SMS service (default)
//! - `mock-services`: Enable mock implementations for testing

// Re-export core types for convenience  
pub use re_core::errors::*;

/// Database module - MySQL implementations using SQLx
#[cfg(feature = "mysql")]
pub mod database;

/// SMS service module - External SMS providers
pub mod sms;

/// Cache module - Redis client and operations  
pub mod cache;

/// Services module - Infrastructure service implementations
pub mod services;

/// Configuration module for infrastructure services
pub mod config {
    //! Configuration management for infrastructure services
    //! 
    //! Handles:
    //! - Database connection strings
    //! - Redis configuration
    //! - SMS service credentials  
    //! - Environment-specific settings
    
    use re_shared::config::{database::DatabaseConfig, cache::CacheConfig};
    use serde::{Deserialize, Serialize};
    
    // Re-export shared configs for backward compatibility
    pub use re_shared::config::{database::DatabaseConfig as InfraDatabaseConfig, cache::CacheConfig as InfraCacheConfig};
    
    /// Infrastructure configuration settings
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct InfrastructureConfig {
        /// Database configuration
        pub database: DatabaseConfig,
        /// Redis cache configuration
        pub cache: CacheConfig,
        /// SMS service configuration
        pub sms: SmsConfig,
    }
    
    /// SMS service configuration
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SmsConfig {
        /// SMS service provider ("twilio", "aws-sns", "mock")
        pub provider: String,
        /// API credentials
        pub api_key: String,
        /// API secret/token
        pub api_secret: String,
        /// From phone number
        pub from_number: String,
    }
    
    impl Default for InfrastructureConfig {
        fn default() -> Self {
            Self {
                database: DatabaseConfig::default(),
                cache: CacheConfig::default(),
                sms: SmsConfig {
                    provider: "mock".to_string(),
                    api_key: String::new(),
                    api_secret: String::new(),
                    from_number: "+1234567890".to_string(),
                },
            }
        }
    }
}

/// Infrastructure service container
#[derive(Clone)]
pub struct InfrastructureServices {
    // Services will be added as modules are implemented
    _marker: std::marker::PhantomData<()>,
}

impl InfrastructureServices {
    /// Create new infrastructure services container
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl Default for InfrastructureServices {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize infrastructure services with async runtime
/// 
/// This function sets up:
/// - Database connection pools
/// - Redis connections
/// - SMS service clients
/// - Tokio async runtime configuration
pub async fn initialize() -> Result<InfrastructureServices, InfrastructureError> {
    tracing::info!("Initializing infrastructure services...");
    
    // Load configuration
    let _config = load_config()?;
    
    // TODO: Initialize database pool
    // TODO: Initialize Redis client
    // TODO: Initialize SMS service
    
    tracing::info!("Infrastructure services initialized successfully");
    
    Ok(InfrastructureServices::new())
}

/// Load infrastructure configuration from environment
fn load_config() -> Result<config::InfrastructureConfig, InfrastructureError> {
    dotenvy::dotenv().ok(); // Load .env file if present
    
    // Use shared config loaders
    let database = re_shared::config::database::DatabaseConfig::from_env();
    let cache = re_shared::config::cache::CacheConfig::from_env();
    
    // Load SMS config (still local to infra)
    let sms = config::SmsConfig {
        provider: std::env::var("SMS_PROVIDER").unwrap_or_else(|_| "mock".to_string()),
        api_key: std::env::var("SMS_API_KEY").unwrap_or_default(),
        api_secret: std::env::var("SMS_API_SECRET").unwrap_or_default(),
        from_number: std::env::var("SMS_FROM_NUMBER").unwrap_or_else(|_| "+1234567890".to_string()),
    };
    
    Ok(config::InfrastructureConfig {
        database,
        cache,
        sms,
    })
}

/// Infrastructure-specific error types
#[derive(Debug, thiserror::Error)]
pub enum InfrastructureError {
    /// Database connection error
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    /// Redis cache error
    #[error("Cache error: {0}")]
    Cache(#[from] redis::RedisError),
    
    /// HTTP request error for external services
    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// SMS service error
    #[error("SMS service error: {0}")]
    Sms(String),
    
    /// General infrastructure error
    #[error("Infrastructure error: {0}")]
    General(String),
}