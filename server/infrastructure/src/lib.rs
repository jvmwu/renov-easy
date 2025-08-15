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
pub use renov_core::errors::*;

/// Database module - MySQL implementations using SQLx
pub mod database;

/// SMS service module - External SMS providers
pub mod sms {
    //! SMS service integrations for verification codes
    //! 
    //! Supports:
    //! - Twilio SMS API
    //! - AWS SNS (future)
    //! - Mock implementation for testing
}

/// Cache module - Redis client and operations  
pub mod cache {
    //! Redis caching layer for performance and rate limiting
    //! 
    //! Provides:
    //! - Redis connection pooling
    //! - Cache operations (get, set, delete, expire)
    //! - Rate limiting counters
    //! - Session storage
}

/// Configuration module for infrastructure services
pub mod config {
    //! Configuration management for infrastructure services
    //! 
    //! Handles:
    //! - Database connection strings
    //! - Redis configuration
    //! - SMS service credentials  
    //! - Environment-specific settings
    
    use serde::{Deserialize, Serialize};
    
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
    
    /// Database configuration
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DatabaseConfig {
        /// Database connection URL
        pub url: String,
        /// Maximum number of connections in pool
        pub max_connections: u32,
        /// Connection timeout in seconds
        pub connect_timeout: u64,
    }
    
    /// Redis cache configuration
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CacheConfig {
        /// Redis connection URL
        pub url: String,
        /// Connection pool size
        pub pool_size: u32,
        /// Default TTL for cache entries in seconds
        pub default_ttl: u64,
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
                database: DatabaseConfig {
                    url: "mysql://localhost:3306/renoveasty".to_string(),
                    max_connections: 10,
                    connect_timeout: 30,
                },
                cache: CacheConfig {
                    url: "redis://localhost:6379".to_string(),
                    pool_size: 10,
                    default_ttl: 3600, // 1 hour
                },
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
    
    // For now, use default config
    // TODO: Load from environment variables and config files
    Ok(config::InfrastructureConfig::default())
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_infrastructure_initialization() {
        let services = initialize().await;
        assert!(services.is_ok());
    }
    
    #[test]
    fn test_default_config() {
        let config = config::InfrastructureConfig::default();
        assert!(!config.database.url.is_empty());
        assert!(config.database.max_connections > 0);
        assert!(!config.cache.url.is_empty());
    }
}