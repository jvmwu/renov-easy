//! Configuration management module for the API server
//! 
//! This module provides a centralized configuration management system that:
//! - Reads configuration from environment variables
//! - Validates required configuration items
//! - Provides sensible defaults for optional configuration
//! - Integrates with shared configuration types

use shared::config::{
    auth::AuthConfig,
    cache::{CacheConfig, CacheStrategyConfig},
    database::DatabaseConfig,
    environment::{Environment, LoggingConfig, MonitoringConfig},
    rate_limit::RateLimitConfig,
    server::{CorsConfig, ServerConfig},
};
use serde::{Deserialize, Serialize};
use std::{env, fmt, error::Error};

/// Configuration errors
#[derive(Debug)]
pub enum ConfigError {
    MissingVar(String),
    InvalidValue { key: String, value: String },
    EnvError(env::VarError),
    ValidationError(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::MissingVar(var) => write!(f, "Missing required environment variable: {}", var),
            ConfigError::InvalidValue { key, value } => write!(f, "Invalid environment variable value for {}: {}", key, value),
            ConfigError::EnvError(e) => write!(f, "Environment variable error: {}", e),
            ConfigError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConfigError::EnvError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<env::VarError> for ConfigError {
    fn from(err: env::VarError) -> Self {
        ConfigError::EnvError(err)
    }
}

/// SMS service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsConfig {
    /// SMS service provider (e.g., "mock", "twilio", "aliyun")
    pub provider: String,
    
    /// API key for the SMS service
    pub api_key: Option<String>,
    
    /// API secret for the SMS service (if required)
    pub api_secret: Option<String>,
    
    /// SMS sender ID or phone number
    pub sender_id: Option<String>,
    
    /// Template ID for SMS messages (provider-specific)
    pub template_id: Option<String>,
    
    /// Enable SMS delivery in production
    pub enabled: bool,
    
    /// Use mock provider in development
    pub use_mock_in_dev: bool,
}

impl Default for SmsConfig {
    fn default() -> Self {
        Self {
            provider: "mock".to_string(),
            api_key: None,
            api_secret: None,
            sender_id: None,
            template_id: None,
            enabled: true,
            use_mock_in_dev: true,
        }
    }
}

impl SmsConfig {
    /// Create from environment variables
    pub fn from_env() -> Self {
        let provider = env::var("SMS_PROVIDER")
            .unwrap_or_else(|_| "mock".to_string());
        let api_key = env::var("SMS_API_KEY").ok();
        let api_secret = env::var("SMS_API_SECRET").ok();
        let sender_id = env::var("SMS_SENDER_ID").ok();
        let template_id = env::var("SMS_TEMPLATE_ID").ok();
        let enabled = env::var("SMS_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);
        let use_mock_in_dev = env::var("SMS_USE_MOCK_IN_DEV")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);
            
        Self {
            provider,
            api_key,
            api_secret,
            sender_id,
            template_id,
            enabled,
            use_mock_in_dev,
        }
    }
    
    /// Check if using mock provider
    pub fn is_mock(&self) -> bool {
        self.provider == "mock"
    }
    
    /// Validate SMS configuration
    pub fn validate(&self, environment: Environment) -> Result<(), ConfigError> {
        // In production, require real SMS configuration unless explicitly using mock
        if environment.is_production() && !self.is_mock() {
            if self.api_key.is_none() {
                return Err(ConfigError::MissingVar("SMS_API_KEY".to_string()));
            }
            // Some providers may require additional fields
            match self.provider.as_str() {
                "twilio" => {
                    if self.api_secret.is_none() {
                        return Err(ConfigError::MissingVar("SMS_API_SECRET".to_string()));
                    }
                    if self.sender_id.is_none() {
                        return Err(ConfigError::MissingVar("SMS_SENDER_ID".to_string()));
                    }
                }
                "aliyun" => {
                    if self.template_id.is_none() {
                        return Err(ConfigError::MissingVar("SMS_TEMPLATE_ID".to_string()));
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

/// Complete application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Environment (development, staging, production)
    pub environment: Environment,
    
    /// Database configuration
    pub database: DatabaseConfig,
    
    /// Cache configuration
    pub cache: CacheStrategyConfig,
    
    /// Authentication configuration
    pub auth: AuthConfig,
    
    /// Server configuration
    pub server: ServerConfig,
    
    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,
    
    /// CORS configuration
    pub cors: CorsConfig,
    
    /// SMS service configuration
    pub sms: SmsConfig,
    
    /// Logging configuration
    pub logging: LoggingConfig,
    
    /// Monitoring configuration
    pub monitoring: MonitoringConfig,
    
    /// Optional Google Maps API key for location services
    pub google_maps_api_key: Option<String>,
}

impl Config {
    /// Create configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        let environment = Environment::from_env();
        
        // Create base configuration based on environment
        let mut config = match environment {
            Environment::Development => Self::development(),
            Environment::Staging => Self::staging(),
            Environment::Production => Self::production()?,
        };
        
        // Override with environment variables if present
        config.override_from_env()?;
        
        // Validate the final configuration
        config.validate()?;
        
        Ok(config)
    }
    
    /// Create development configuration with defaults
    fn development() -> Self {
        let environment = Environment::Development;
        Self {
            environment,
            database: DatabaseConfig::new("mysql://localhost:3306/renoveasy_dev"),
            cache: CacheStrategyConfig::default(),
            auth: AuthConfig::default(),
            server: ServerConfig::default(),
            rate_limit: RateLimitConfig::development(),
            cors: CorsConfig::development(),
            sms: SmsConfig {
                provider: "mock".to_string(),
                use_mock_in_dev: true,
                ..Default::default()
            },
            logging: LoggingConfig::for_environment(environment),
            monitoring: MonitoringConfig::default(),
            google_maps_api_key: None,
        }
    }
    
    /// Create staging configuration
    fn staging() -> Self {
        let environment = Environment::Staging;
        let mut config = Self::development();
        config.environment = environment;
        config.logging = LoggingConfig::for_environment(environment);
        config.database = DatabaseConfig::new("mysql://staging-db:3306/renoveasy_staging")
            .with_max_connections(20);
        config
    }
    
    /// Create production configuration (requires certain environment variables)
    fn production() -> Result<Self, ConfigError> {
        let environment = Environment::Production;
        
        // In production, certain configurations are required
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| ConfigError::MissingVar("DATABASE_URL".to_string()))?;
        let redis_url = env::var("REDIS_URL")
            .map_err(|_| ConfigError::MissingVar("REDIS_URL".to_string()))?;
        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| ConfigError::MissingVar("JWT_SECRET".to_string()))?;
        
        Ok(Self {
            environment,
            database: DatabaseConfig::new(database_url)
                .with_max_connections(50),
            cache: CacheStrategyConfig {
                enabled: true,
                cache_type: shared::config::cache::CacheType::Redis,
                redis: Some(CacheConfig::new(redis_url)),
                memory: None,
            },
            auth: AuthConfig {
                jwt: shared::config::auth::JwtConfig::new(jwt_secret),
                session: shared::config::auth::SessionConfig {
                    secure: true,
                    ..Default::default()
                },
                oauth2: None,
            },
            server: ServerConfig::new("0.0.0.0", 8080),
            rate_limit: RateLimitConfig::production(),
            cors: CorsConfig::default(),
            sms: SmsConfig::from_env(),
            logging: LoggingConfig::for_environment(environment),
            monitoring: MonitoringConfig {
                metrics_enabled: true,
                health_enabled: true,
                tracing_enabled: true,
                ..Default::default()
            },
            google_maps_api_key: env::var("GOOGLE_MAPS_API_KEY").ok(),
        })
    }
    
    /// Override configuration with environment variables
    fn override_from_env(&mut self) -> Result<(), ConfigError> {
        // Override database configuration
        if let Ok(url) = env::var("DATABASE_URL") {
            self.database.url = url;
        }
        if let Ok(max_conn) = env::var("DATABASE_MAX_CONNECTIONS") {
            self.database.max_connections = max_conn.parse()
                .map_err(|_| ConfigError::InvalidValue {
                    key: "DATABASE_MAX_CONNECTIONS".to_string(),
                    value: max_conn,
                })?;
        }
        
        // Override cache configuration
        if let Some(redis) = &mut self.cache.redis {
            if let Ok(url) = env::var("REDIS_URL") {
                redis.url = url;
            }
        }
        
        // Override JWT configuration
        if let Ok(secret) = env::var("JWT_SECRET") {
            self.auth.jwt.secret = secret;
        }
        if let Ok(expiry) = env::var("JWT_ACCESS_TOKEN_EXPIRY") {
            self.auth.jwt.access_token_expiry = expiry.parse()
                .map_err(|_| ConfigError::InvalidValue {
                    key: "JWT_ACCESS_TOKEN_EXPIRY".to_string(),
                    value: expiry,
                })?;
        }
        if let Ok(expiry) = env::var("JWT_REFRESH_TOKEN_EXPIRY") {
            self.auth.jwt.refresh_token_expiry = expiry.parse()
                .map_err(|_| ConfigError::InvalidValue {
                    key: "JWT_REFRESH_TOKEN_EXPIRY".to_string(),
                    value: expiry,
                })?;
        }
        
        // Override server configuration
        if let Ok(host) = env::var("SERVER_HOST") {
            self.server.host = host;
        }
        if let Ok(port) = env::var("SERVER_PORT") {
            self.server.port = port.parse()
                .map_err(|_| ConfigError::InvalidValue {
                    key: "SERVER_PORT".to_string(),
                    value: port,
                })?;
        }
        
        // Override SMS configuration
        self.sms = SmsConfig::from_env();
        
        // Override Google Maps API key
        if let Ok(key) = env::var("GOOGLE_MAPS_API_KEY") {
            self.google_maps_api_key = Some(key);
        }
        
        Ok(())
    }
    
    /// Validate the complete configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate JWT configuration
        if self.environment.is_production() && self.auth.jwt.is_using_default_secret() {
            return Err(ConfigError::ValidationError(
                "JWT secret must be changed in production".to_string()
            ));
        }
        
        // Validate database configuration
        if self.environment.is_production() && !self.database.is_production() {
            return Err(ConfigError::ValidationError(
                "Database URL appears to be localhost in production".to_string()
            ));
        }
        
        // Validate SMS configuration
        self.sms.validate(self.environment)?;
        
        // Validate rate limiting is enabled in production
        if self.environment.is_production() && !self.rate_limit.enabled {
            return Err(ConfigError::ValidationError(
                "Rate limiting should be enabled in production".to_string()
            ));
        }
        
        Ok(())
    }
    
    // Backward compatibility methods
    
    pub fn is_development(&self) -> bool {
        self.environment.is_development()
    }

    pub fn is_production(&self) -> bool {
        self.environment.is_production()
    }
    
    pub fn database_url(&self) -> &str {
        &self.database.url
    }
    
    pub fn redis_url(&self) -> &str {
        self.cache.redis.as_ref()
            .map(|c| c.url.as_str())
            .unwrap_or("redis://localhost:6379")
    }
    
    pub fn jwt_secret(&self) -> &str {
        &self.auth.jwt.secret
    }
    
    pub fn jwt_access_token_expiry(&self) -> i64 {
        self.auth.jwt.access_token_expiry
    }
    
    pub fn jwt_refresh_token_expiry(&self) -> i64 {
        self.auth.jwt.refresh_token_expiry
    }
    
    pub fn server_host(&self) -> &str {
        &self.server.host
    }
    
    pub fn server_port(&self) -> u16 {
        self.server.port
    }
    
    // Additional backward compatibility for SMS
    pub fn sms_provider(&self) -> &str {
        &self.sms.provider
    }
    
    pub fn sms_api_key(&self) -> Option<&str> {
        self.sms.api_key.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_development_config() {
        let config = Config::development();
        assert!(config.is_development());
        assert!(!config.is_production());
        assert_eq!(config.sms.provider, "mock");
        assert!(config.sms.use_mock_in_dev);
    }
    
    #[test]
    fn test_sms_config_default() {
        let sms = SmsConfig::default();
        assert_eq!(sms.provider, "mock");
        assert!(sms.enabled);
        assert!(sms.use_mock_in_dev);
        assert!(sms.is_mock());
    }
    
    #[test]
    fn test_config_validation_development() {
        let config = Config::development();
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_backward_compatibility() {
        let config = Config::development();
        assert_eq!(config.database_url(), "mysql://localhost:3306/renoveasy_dev");
        assert_eq!(config.redis_url(), "redis://localhost:6379");
        assert_eq!(config.server_host(), "127.0.0.1");
        assert_eq!(config.server_port(), 8080);
        assert_eq!(config.sms_provider(), "mock");
        assert_eq!(config.sms_api_key(), None);
    }
}