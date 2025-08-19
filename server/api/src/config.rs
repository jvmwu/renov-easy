use shared::config::{
    auth::AuthConfig,
    cache::CacheConfig,
    database::DatabaseConfig,
    environment::Environment,
    server::ServerConfig,
};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub cache: CacheConfig,
    pub auth: AuthConfig,
    pub server: ServerConfig,
    pub sms_provider: String,
    pub sms_api_key: Option<String>,
    pub google_maps_api_key: Option<String>,
    pub environment: Environment,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        let environment = Environment::from_env();

        Ok(Config {
            database: DatabaseConfig::from_env(),
            cache: CacheConfig::from_env(),
            auth: AuthConfig::from_env(),
            server: ServerConfig::from_env(),
            sms_provider: env::var("SMS_PROVIDER")
                .unwrap_or_else(|_| "mock".to_string()),
            sms_api_key: env::var("SMS_API_KEY").ok(),
            google_maps_api_key: env::var("GOOGLE_MAPS_API_KEY").ok(),
            environment,
        })
    }

    pub fn is_development(&self) -> bool {
        self.environment.is_development()
    }

    pub fn is_production(&self) -> bool {
        self.environment.is_production()
    }
    
    // Backward compatibility methods
    pub fn database_url(&self) -> &str {
        &self.database.url
    }
    
    pub fn redis_url(&self) -> &str {
        &self.cache.url
    }
    
    pub fn jwt_secret(&self) -> &str {
        self.auth.jwt_secret()
    }
    
    pub fn jwt_access_token_expiry(&self) -> i64 {
        self.auth.access_token_expiry_seconds()
    }
    
    pub fn jwt_refresh_token_expiry(&self) -> i64 {
        self.auth.refresh_token_expiry_seconds()
    }
    
    pub fn server_host(&self) -> &str {
        &self.server.host
    }
    
    pub fn server_port(&self) -> u16 {
        self.server.port
    }
}