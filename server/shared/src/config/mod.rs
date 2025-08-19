//! Configuration module with business-specific sub-modules
//!
//! This module organizes configuration into logical business areas:
//! - `auth` - Authentication and authorization configuration
//! - `cache` - Caching strategy and Redis configuration
//! - `database` - Database connection and pool configuration
//! - `environment` - Environment detection and logging configuration
//! - `rate_limit` - Rate limiting for APIs, SMS, and authentication
//! - `server` - HTTP server, CORS, and TLS configuration

pub mod auth;
pub mod cache;
pub mod database;
pub mod environment;
pub mod rate_limit;
pub mod server;

use serde::{Deserialize, Serialize};

// Re-export commonly used types
pub use auth::{AuthConfig, JwtConfig, SessionConfig};
pub use cache::{CacheConfig, CacheStrategyConfig, CacheType};
pub use database::DatabaseConfig;
pub use environment::{Environment, LoggingConfig, MonitoringConfig};
pub use rate_limit::RateLimitConfig;
pub use server::{CorsConfig, ServerConfig, TlsConfig};

/// Complete application configuration combining all sub-configurations
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    /// Environment configuration
    pub environment: Environment,
    
    /// Server configuration
    pub server: ServerConfig,
    
    /// Database configuration
    pub database: DatabaseConfig,
    
    /// Authentication configuration
    pub auth: AuthConfig,
    
    /// Cache configuration
    pub cache: CacheStrategyConfig,
    
    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,
    
    /// CORS configuration
    #[serde(default)]
    pub cors: CorsConfig,
    
    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,
    
    /// Monitoring configuration
    #[serde(default)]
    pub monitoring: MonitoringConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        let env = Environment::default();
        Self {
            environment: env,
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            auth: AuthConfig {
                jwt: JwtConfig::default(),
                session: SessionConfig::default(),
                oauth2: None,
            },
            cache: CacheStrategyConfig::default(),
            rate_limit: RateLimitConfig::default(),
            cors: CorsConfig::default(),
            logging: LoggingConfig::for_environment(env),
            monitoring: MonitoringConfig::default(),
        }
    }
}

impl AppConfig {
    /// Create configuration for development environment
    pub fn development() -> Self {
        Self {
            environment: Environment::Development,
            server: ServerConfig::default(),
            database: DatabaseConfig::new("mysql://localhost:3306/renoveasy_dev"),
            auth: AuthConfig {
                jwt: JwtConfig::default(),
                session: SessionConfig::default(),
                oauth2: None,
            },
            cache: CacheStrategyConfig::default(),
            rate_limit: RateLimitConfig::development(),
            cors: CorsConfig::development(),
            logging: LoggingConfig::for_environment(Environment::Development),
            monitoring: MonitoringConfig::default(),
        }
    }
    
    /// Create configuration for production environment
    pub fn production() -> Self {
        Self {
            environment: Environment::Production,
            server: ServerConfig::new("0.0.0.0", 8080),
            database: DatabaseConfig::new("mysql://prod-db:3306/renoveasy")
                .with_max_connections(50),
            auth: AuthConfig {
                jwt: JwtConfig::new("use-env-variable"),
                session: SessionConfig {
                    secure: true,
                    ..Default::default()
                },
                oauth2: None,
            },
            cache: CacheStrategyConfig::default(),
            rate_limit: RateLimitConfig::production(),
            cors: CorsConfig::default(),
            logging: LoggingConfig::for_environment(Environment::Production),
            monitoring: MonitoringConfig {
                metrics_enabled: true,
                health_enabled: true,
                tracing_enabled: true,
                ..Default::default()
            },
        }
    }
    
    /// Load configuration from environment
    pub fn from_env() -> Self {
        let env = Environment::from_env();
        match env {
            Environment::Development => Self::development(),
            Environment::Production => Self::production(),
            Environment::Staging => {
                let mut config = Self::development();
                config.environment = Environment::Staging;
                config.logging = LoggingConfig::for_environment(Environment::Staging);
                config
            }
        }
    }
}