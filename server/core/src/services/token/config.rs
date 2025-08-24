//! Configuration for the token service

use jsonwebtoken::Algorithm;
use re_shared::config::auth::AuthConfig;
use super::key_manager::{Rs256KeyManager, Rs256KeyConfig};
use crate::errors::DomainError;

/// Configuration for the token service
#[derive(Debug, Clone)]
pub struct TokenServiceConfig {
    /// JWT signing secret (for HS256, deprecated)
    pub jwt_secret: String,
    /// JWT signing algorithm
    pub algorithm: Algorithm,
    /// Access token expiry in minutes
    pub access_token_expiry_minutes: i64,
    /// Refresh token expiry in days
    pub refresh_token_expiry_days: i64,
    /// RS256 key configuration (optional, for RS256 algorithm)
    pub rs256_config: Option<Rs256KeyConfig>,
}

impl Default for TokenServiceConfig {
    fn default() -> Self {
        let auth_config = AuthConfig::default();
        // Default to RS256 algorithm with key files
        Self {
            jwt_secret: auth_config.jwt_secret().to_string(),
            algorithm: Algorithm::RS256,
            access_token_expiry_minutes: auth_config.access_token_expiry_seconds() / 60,
            refresh_token_expiry_days: auth_config.refresh_token_expiry_seconds() / (60 * 60 * 24),
            rs256_config: Some(Rs256KeyConfig::default()),
        }
    }
}

impl From<AuthConfig> for TokenServiceConfig {
    fn from(config: AuthConfig) -> Self {
        // Use RS256 by default with key configuration from environment
        let algorithm = std::env::var("JWT_ALGORITHM")
            .unwrap_or_else(|_| "RS256".to_string());
        
        let algorithm = match algorithm.as_str() {
            "RS256" => Algorithm::RS256,
            "HS256" => Algorithm::HS256,
            _ => Algorithm::RS256, // Default to RS256 for security
        };
        
        let rs256_config = if algorithm == Algorithm::RS256 {
            Some(Rs256KeyConfig::from_env())
        } else {
            None
        };
        
        Self {
            jwt_secret: config.jwt_secret().to_string(),
            algorithm,
            access_token_expiry_minutes: config.access_token_expiry_seconds() / 60,
            refresh_token_expiry_days: config.refresh_token_expiry_seconds() / (60 * 60 * 24),
            rs256_config,
        }
    }
}

impl TokenServiceConfig {
    /// Creates a new configuration with RS256 algorithm
    pub fn with_rs256(mut self) -> Self {
        self.algorithm = Algorithm::RS256;
        self.rs256_config = Some(Rs256KeyConfig::default());
        self
    }
    
    /// Creates a new configuration with custom RS256 key paths
    pub fn with_rs256_keys(mut self, private_key_path: String, public_key_path: String) -> Self {
        self.algorithm = Algorithm::RS256;
        self.rs256_config = Some(Rs256KeyConfig {
            private_key_path,
            public_key_path,
            allow_rotation: false,
            rotation_check_interval: 0,
        });
        self
    }
    
    /// Loads RS256 key manager if configured for RS256
    pub fn load_key_manager(&self) -> Result<Option<Rs256KeyManager>, DomainError> {
        if self.algorithm != Algorithm::RS256 {
            return Ok(None);
        }
        
        let config = self.rs256_config.as_ref()
            .ok_or_else(|| DomainError::Internal {
                message: "RS256 algorithm requires key configuration".to_string(),
            })?;
        
        let manager = Rs256KeyManager::new(
            &config.private_key_path,
            &config.public_key_path,
        )?;
        
        Ok(Some(manager))
    }
}