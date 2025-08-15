//! Token entities for JWT-based authentication.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Access token expiration time (15 minutes)
pub const ACCESS_TOKEN_EXPIRY_MINUTES: i64 = 15;

/// Refresh token expiration time (7 days)
pub const REFRESH_TOKEN_EXPIRY_DAYS: i64 = 7;

/// JWT issuer
pub const JWT_ISSUER: &str = "renov-easy";

/// JWT audience
pub const JWT_AUDIENCE: &str = "renov-easy-api";

/// Claims structure for JWT payload
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    
    /// Issued at timestamp
    pub iat: i64,
    
    /// Expiration timestamp
    pub exp: i64,
    
    /// Not before timestamp
    pub nbf: i64,
    
    /// Issuer
    pub iss: String,
    
    /// Audience
    pub aud: String,
    
    /// JWT ID (unique identifier for the token)
    pub jti: String,
    
    /// User type (if set)
    pub user_type: Option<String>,
    
    /// Whether the user is verified
    pub is_verified: bool,
}

impl Claims {
    /// Creates new claims for an access token
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user's UUID
    /// * `user_type` - The user's type (Customer or Worker)
    /// * `is_verified` - Whether the user is verified
    ///
    /// # Returns
    ///
    /// A new `Claims` instance for an access token
    pub fn new_access_token(
        user_id: Uuid,
        user_type: Option<String>,
        is_verified: bool,
    ) -> Self {
        let now = Utc::now();
        let expiry = now + Duration::minutes(ACCESS_TOKEN_EXPIRY_MINUTES);
        
        Self {
            sub: user_id.to_string(),
            iat: now.timestamp(),
            exp: expiry.timestamp(),
            nbf: now.timestamp(),
            iss: JWT_ISSUER.to_string(),
            aud: JWT_AUDIENCE.to_string(),
            jti: Uuid::new_v4().to_string(),
            user_type,
            is_verified,
        }
    }
    
    /// Creates new claims for a refresh token
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user's UUID
    ///
    /// # Returns
    ///
    /// A new `Claims` instance for a refresh token
    pub fn new_refresh_token(user_id: Uuid) -> Self {
        let now = Utc::now();
        let expiry = now + Duration::days(REFRESH_TOKEN_EXPIRY_DAYS);
        
        Self {
            sub: user_id.to_string(),
            iat: now.timestamp(),
            exp: expiry.timestamp(),
            nbf: now.timestamp(),
            iss: JWT_ISSUER.to_string(),
            aud: JWT_AUDIENCE.to_string(),
            jti: Uuid::new_v4().to_string(),
            user_type: None,
            is_verified: false,
        }
    }
    
    /// Checks if the claims have expired
    ///
    /// # Returns
    ///
    /// `true` if the claims have expired, `false` otherwise
    pub fn is_expired(&self) -> bool {
        let now = Utc::now().timestamp();
        now >= self.exp
    }
    
    /// Checks if the claims are valid
    ///
    /// # Returns
    ///
    /// `true` if the claims are valid (not expired and after nbf), `false` otherwise
    pub fn is_valid(&self) -> bool {
        let now = Utc::now().timestamp();
        now >= self.nbf && now < self.exp
    }
    
    /// Gets the user ID from the claims
    ///
    /// # Returns
    ///
    /// `Ok(Uuid)` if the subject can be parsed as a UUID, `Err` otherwise
    pub fn user_id(&self) -> Result<Uuid, uuid::Error> {
        Uuid::parse_str(&self.sub)
    }
}

/// Refresh token entity stored in the database
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefreshToken {
    /// Unique identifier for the refresh token
    pub id: Uuid,
    
    /// User ID this token belongs to
    pub user_id: Uuid,
    
    /// Hashed token value for security
    pub token_hash: String,
    
    /// Timestamp when the token was created
    pub created_at: DateTime<Utc>,
    
    /// Timestamp when the token expires
    pub expires_at: DateTime<Utc>,
    
    /// Whether the token has been revoked
    pub is_revoked: bool,
}

impl RefreshToken {
    /// Creates a new refresh token
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user's UUID
    /// * `token_hash` - The hashed token value
    ///
    /// # Returns
    ///
    /// A new `RefreshToken` instance
    pub fn new(user_id: Uuid, token_hash: String) -> Self {
        let now = Utc::now();
        let expires_at = now + Duration::days(REFRESH_TOKEN_EXPIRY_DAYS);
        
        Self {
            id: Uuid::new_v4(),
            user_id,
            token_hash,
            created_at: now,
            expires_at,
            is_revoked: false,
        }
    }
    
    /// Checks if the refresh token has expired
    ///
    /// # Returns
    ///
    /// `true` if the token has expired, `false` otherwise
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
    
    /// Checks if the refresh token is valid
    ///
    /// A token is valid if it hasn't expired and hasn't been revoked
    ///
    /// # Returns
    ///
    /// `true` if the token is valid, `false` otherwise
    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.is_revoked
    }
    
    /// Revokes the refresh token
    pub fn revoke(&mut self) {
        self.is_revoked = true;
    }
    
    /// Gets the time remaining until expiration
    ///
    /// # Returns
    ///
    /// A `Duration` representing the time until expiration, or zero if expired
    pub fn time_until_expiration(&self) -> Duration {
        let now = Utc::now();
        if self.expires_at > now {
            self.expires_at - now
        } else {
            Duration::zero()
        }
    }
}

/// Token pair returned to the client
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenPair {
    /// JWT access token
    pub access_token: String,
    
    /// JWT refresh token
    pub refresh_token: String,
    
    /// Access token expiry time in seconds
    pub access_expires_in: i64,
    
    /// Refresh token expiry time in seconds
    pub refresh_expires_in: i64,
}

impl TokenPair {
    /// Creates a new token pair
    ///
    /// # Arguments
    ///
    /// * `access_token` - The JWT access token
    /// * `refresh_token` - The JWT refresh token
    ///
    /// # Returns
    ///
    /// A new `TokenPair` instance with calculated expiry times
    pub fn new(access_token: String, refresh_token: String) -> Self {
        Self {
            access_token,
            refresh_token,
            access_expires_in: ACCESS_TOKEN_EXPIRY_MINUTES * 60,
            refresh_expires_in: REFRESH_TOKEN_EXPIRY_DAYS * 24 * 60 * 60,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_access_token_claims() {
        let user_id = Uuid::new_v4();
        let claims = Claims::new_access_token(
            user_id,
            Some("customer".to_string()),
            true,
        );
        
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.iss, JWT_ISSUER);
        assert_eq!(claims.aud, JWT_AUDIENCE);
        assert_eq!(claims.user_type, Some("customer".to_string()));
        assert!(claims.is_verified);
        assert!(claims.is_valid());
        assert!(!claims.is_expired());
    }
    
    #[test]
    fn test_refresh_token_claims() {
        let user_id = Uuid::new_v4();
        let claims = Claims::new_refresh_token(user_id);
        
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.iss, JWT_ISSUER);
        assert_eq!(claims.aud, JWT_AUDIENCE);
        assert_eq!(claims.user_type, None);
        assert!(!claims.is_verified);
        assert!(claims.is_valid());
        assert!(!claims.is_expired());
    }
    
    #[test]
    fn test_claims_user_id_parsing() {
        let user_id = Uuid::new_v4();
        let claims = Claims::new_access_token(user_id, None, false);
        
        let parsed_id = claims.user_id().unwrap();
        assert_eq!(parsed_id, user_id);
    }
    
    #[test]
    fn test_claims_expiration() {
        let user_id = Uuid::new_v4();
        let mut claims = Claims::new_access_token(user_id, None, false);
        
        // Set expiration to past
        claims.exp = Utc::now().timestamp() - 1;
        
        assert!(claims.is_expired());
        assert!(!claims.is_valid());
    }
    
    #[test]
    fn test_claims_not_before() {
        let user_id = Uuid::new_v4();
        let mut claims = Claims::new_access_token(user_id, None, false);
        
        // Set nbf to future
        claims.nbf = Utc::now().timestamp() + 3600;
        
        assert!(!claims.is_valid());
    }
    
    #[test]
    fn test_refresh_token_creation() {
        let user_id = Uuid::new_v4();
        let token_hash = "hashed_token_value".to_string();
        let token = RefreshToken::new(user_id, token_hash.clone());
        
        assert_eq!(token.user_id, user_id);
        assert_eq!(token.token_hash, token_hash);
        assert!(!token.is_revoked);
        assert!(!token.is_expired());
        assert!(token.is_valid());
    }
    
    #[test]
    fn test_refresh_token_revocation() {
        let user_id = Uuid::new_v4();
        let mut token = RefreshToken::new(user_id, "hash".to_string());
        
        assert!(token.is_valid());
        
        token.revoke();
        
        assert!(token.is_revoked);
        assert!(!token.is_valid());
    }
    
    #[test]
    fn test_refresh_token_expiration() {
        let user_id = Uuid::new_v4();
        let mut token = RefreshToken::new(user_id, "hash".to_string());
        
        // Manually set expiration to past
        token.expires_at = Utc::now() - Duration::days(1);
        
        assert!(token.is_expired());
        assert!(!token.is_valid());
    }
    
    #[test]
    fn test_refresh_token_time_until_expiration() {
        let user_id = Uuid::new_v4();
        let token = RefreshToken::new(user_id, "hash".to_string());
        
        let time_remaining = token.time_until_expiration();
        let expected_max = Duration::days(REFRESH_TOKEN_EXPIRY_DAYS);
        let expected_min = Duration::days(REFRESH_TOKEN_EXPIRY_DAYS - 1);
        
        assert!(time_remaining <= expected_max);
        assert!(time_remaining > expected_min);
    }
    
    #[test]
    fn test_token_pair_creation() {
        let access = "access_token_jwt".to_string();
        let refresh = "refresh_token_jwt".to_string();
        let pair = TokenPair::new(access.clone(), refresh.clone());
        
        assert_eq!(pair.access_token, access);
        assert_eq!(pair.refresh_token, refresh);
        assert_eq!(pair.access_expires_in, ACCESS_TOKEN_EXPIRY_MINUTES * 60);
        assert_eq!(pair.refresh_expires_in, REFRESH_TOKEN_EXPIRY_DAYS * 24 * 60 * 60);
    }
    
    #[test]
    fn test_token_pair_serialization() {
        let pair = TokenPair::new(
            "access_token".to_string(),
            "refresh_token".to_string(),
        );
        
        // Serialize to JSON
        let json = serde_json::to_string(&pair).unwrap();
        
        // Deserialize back
        let deserialized: TokenPair = serde_json::from_str(&json).unwrap();
        
        assert_eq!(pair, deserialized);
    }
    
    #[test]
    fn test_claims_serialization() {
        let user_id = Uuid::new_v4();
        let claims = Claims::new_access_token(
            user_id,
            Some("worker".to_string()),
            true,
        );
        
        // Serialize to JSON
        let json = serde_json::to_string(&claims).unwrap();
        
        // Deserialize back
        let deserialized: Claims = serde_json::from_str(&json).unwrap();
        
        assert_eq!(claims, deserialized);
    }
    
    #[test]
    fn test_refresh_token_serialization() {
        let user_id = Uuid::new_v4();
        let token = RefreshToken::new(user_id, "token_hash".to_string());
        
        // Serialize to JSON
        let json = serde_json::to_string(&token).unwrap();
        
        // Deserialize back
        let deserialized: RefreshToken = serde_json::from_str(&json).unwrap();
        
        assert_eq!(token, deserialized);
    }
}