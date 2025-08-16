//! Configuration for the token service

use jsonwebtoken::Algorithm;

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
        Self {
            jwt_secret: "development-secret-please-change-in-production".to_string(),
            algorithm: Algorithm::HS256,
            access_token_expiry_minutes: 15,
            refresh_token_expiry_days: 7,
        }
    }
}