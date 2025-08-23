//! Configuration for the token service

use jsonwebtoken::Algorithm;
use re_shared::config::auth::AuthConfig;

/// Configuration for the token service
#[derive(Debug, Clone)]
pub struct TokenServiceConfig {
    /// JWT signing secret
    pub jwt_secret: String,
    /// JWT signing algorithm
    pub algorithm: Algorithm,
    /// Access token expiry in minutes
    pub access_token_expiry_minutes: i64,
    /// Refresh token expiry in days
    pub refresh_token_expiry_days: i64,
}

impl Default for TokenServiceConfig {
    fn default() -> Self {
        let auth_config = AuthConfig::default();
        Self {
            jwt_secret: auth_config.jwt_secret().to_string(),
            algorithm: Algorithm::HS256,
            access_token_expiry_minutes: auth_config.access_token_expiry_seconds() / 60,
            refresh_token_expiry_days: auth_config.refresh_token_expiry_seconds() / (60 * 60 * 24),
        }
    }
}

impl From<AuthConfig> for TokenServiceConfig {
    fn from(config: AuthConfig) -> Self {
        Self {
            jwt_secret: config.jwt_secret().to_string(),
            algorithm: Algorithm::HS256,
            access_token_expiry_minutes: config.access_token_expiry_seconds() / 60,
            refresh_token_expiry_days: config.refresh_token_expiry_seconds() / (60 * 60 * 24),
        }
    }
}