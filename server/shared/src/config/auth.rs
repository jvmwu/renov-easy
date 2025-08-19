//! Authentication and authorization configuration

use serde::{Deserialize, Serialize};

/// JWT authentication configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JwtConfig {
    /// JWT secret key for signing tokens
    pub secret: String,
    
    /// Access token expiry time in seconds
    pub access_token_expiry: i64,
    
    /// Refresh token expiry time in seconds
    pub refresh_token_expiry: i64,
    
    /// JWT issuer claim
    pub issuer: String,
    
    /// JWT audience claim
    #[serde(default)]
    pub audience: Option<String>,
    
    /// Algorithm for JWT signing (default: HS256)
    #[serde(default = "default_algorithm")]
    pub algorithm: String,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: String::from("your-secret-key-change-in-production"),
            access_token_expiry: 900,     // 15 minutes
            refresh_token_expiry: 604800,  // 7 days
            issuer: String::from("renoveasy"),
            audience: None,
            algorithm: default_algorithm(),
        }
    }
}

impl JwtConfig {
    /// Create a new JWT configuration with secret
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
            ..Default::default()
        }
    }
    
    /// Set access token expiry in minutes
    pub fn with_access_expiry_minutes(mut self, minutes: i64) -> Self {
        self.access_token_expiry = minutes * 60;
        self
    }
    
    /// Set refresh token expiry in days
    pub fn with_refresh_expiry_days(mut self, days: i64) -> Self {
        self.refresh_token_expiry = days * 86400;
        self
    }
    
    /// Check if using default secret (security warning)
    pub fn is_using_default_secret(&self) -> bool {
        self.secret == "your-secret-key-change-in-production"
    }
}

/// OAuth2 provider configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OAuth2Config {
    /// OAuth2 client ID
    pub client_id: String,
    
    /// OAuth2 client secret
    pub client_secret: String,
    
    /// Authorization URL
    pub auth_url: String,
    
    /// Token exchange URL
    pub token_url: String,
    
    /// Redirect URL after authentication
    pub redirect_url: String,
    
    /// OAuth2 scopes
    #[serde(default)]
    pub scopes: Vec<String>,
}

/// Session configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SessionConfig {
    /// Session timeout in seconds
    pub timeout: u64,
    
    /// Session cookie name
    pub cookie_name: String,
    
    /// Session cookie secure flag (HTTPS only)
    pub secure: bool,
    
    /// Session cookie SameSite attribute
    pub same_site: String,
    
    /// Session cookie HttpOnly flag
    #[serde(default = "default_http_only")]
    pub http_only: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            timeout: 3600,  // 1 hour
            cookie_name: String::from("renoveasy_session"),
            secure: false,  // Set to true in production
            same_site: String::from("Lax"),
            http_only: default_http_only(),
        }
    }
}

/// Complete authentication configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    /// JWT configuration
    pub jwt: JwtConfig,
    
    /// Session configuration
    #[serde(default)]
    pub session: SessionConfig,
    
    /// OAuth2 providers (optional)
    #[serde(default)]
    pub oauth2: Option<OAuth2Providers>,
}

impl AuthConfig {
    /// Create from environment variables
    pub fn from_env() -> Self {
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "development-secret-please-change-in-production".to_string());
        let access_token_expiry = std::env::var("JWT_ACCESS_TOKEN_EXPIRY")
            .unwrap_or_else(|_| "900".to_string())
            .parse()
            .unwrap_or(900);
        let refresh_token_expiry = std::env::var("JWT_REFRESH_TOKEN_EXPIRY")
            .unwrap_or_else(|_| "604800".to_string())
            .parse()
            .unwrap_or(604800);
            
        Self {
            jwt: JwtConfig {
                secret: jwt_secret,
                access_token_expiry,
                refresh_token_expiry,
                issuer: String::from("renoveasy"),
                audience: None,
                algorithm: default_algorithm(),
            },
            session: SessionConfig::default(),
            oauth2: None,
        }
    }
    
    /// Get JWT secret (backward compatibility)
    pub fn jwt_secret(&self) -> &str {
        &self.jwt.secret
    }
    
    /// Get access token expiry in seconds (backward compatibility)
    pub fn access_token_expiry_seconds(&self) -> i64 {
        self.jwt.access_token_expiry
    }
    
    /// Get refresh token expiry in seconds (backward compatibility)
    pub fn refresh_token_expiry_seconds(&self) -> i64 {
        self.jwt.refresh_token_expiry
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt: JwtConfig::default(),
            session: SessionConfig::default(),
            oauth2: None,
        }
    }
}

/// OAuth2 provider configurations
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OAuth2Providers {
    #[serde(default)]
    pub google: Option<OAuth2Config>,
    
    #[serde(default)]
    pub wechat: Option<OAuth2Config>,
    
    #[serde(default)]
    pub apple: Option<OAuth2Config>,
}

fn default_algorithm() -> String {
    String::from("HS256")
}

fn default_http_only() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_jwt_config_default() {
        let config = JwtConfig::default();
        assert_eq!(config.access_token_expiry, 900);
        assert_eq!(config.refresh_token_expiry, 604800);
        assert_eq!(config.algorithm, "HS256");
        assert!(config.is_using_default_secret());
    }
    
    #[test]
    fn test_jwt_config_builder() {
        let config = JwtConfig::new("my-secret")
            .with_access_expiry_minutes(30)
            .with_refresh_expiry_days(14);
        
        assert_eq!(config.access_token_expiry, 1800);
        assert_eq!(config.refresh_token_expiry, 1209600);
        assert!(!config.is_using_default_secret());
    }
    
    #[test]
    fn test_session_config_default() {
        let config = SessionConfig::default();
        assert_eq!(config.timeout, 3600);
        assert_eq!(config.cookie_name, "renoveasy_session");
        assert!(config.http_only);
        assert!(!config.secure);
    }
}