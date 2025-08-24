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
    
    /// Hashed phone number for additional verification
    pub phone_hash: Option<String>,
    
    /// Device fingerprint for tracking token usage
    pub device_fingerprint: Option<String>,
    
    /// Token family ID for rotation tracking
    pub token_family: Option<String>,
}

impl Claims {
    /// Creates new claims for an access token
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user's UUID
    /// * `user_type` - The user's type (Customer or Worker)
    /// * `is_verified` - Whether the user is verified
    /// * `phone_hash` - Hashed phone number
    /// * `device_fingerprint` - Device fingerprint for tracking
    ///
    /// # Returns
    ///
    /// A new `Claims` instance for an access token
    pub fn new_access_token(
        user_id: Uuid,
        user_type: Option<String>,
        is_verified: bool,
        phone_hash: Option<String>,
        device_fingerprint: Option<String>,
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
            phone_hash,
            device_fingerprint,
            token_family: None,
        }
    }
    
    /// Creates new claims for a refresh token
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user's UUID
    /// * `token_family` - Token family ID for rotation tracking
    /// * `device_fingerprint` - Device fingerprint for tracking
    ///
    /// # Returns
    ///
    /// A new `Claims` instance for a refresh token
    pub fn new_refresh_token(
        user_id: Uuid,
        token_family: Option<String>,
        device_fingerprint: Option<String>,
    ) -> Self {
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
            phone_hash: None,
            device_fingerprint,
            token_family,
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
    
    /// Token family ID for rotation tracking
    pub token_family: Option<String>,
    
    /// Device fingerprint for security tracking
    pub device_fingerprint: Option<String>,
    
    /// Previous token ID in the rotation chain
    pub previous_token_id: Option<Uuid>,
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
        Self::new_with_metadata(user_id, token_hash, None, None, None)
    }
    
    /// Creates a new refresh token with metadata
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user's UUID
    /// * `token_hash` - The hashed token value
    /// * `token_family` - Token family ID for rotation
    /// * `device_fingerprint` - Device fingerprint
    /// * `previous_token_id` - Previous token in rotation chain
    ///
    /// # Returns
    ///
    /// A new `RefreshToken` instance with metadata
    pub fn new_with_metadata(
        user_id: Uuid,
        token_hash: String,
        token_family: Option<String>,
        device_fingerprint: Option<String>,
        previous_token_id: Option<Uuid>,
    ) -> Self {
        let now = Utc::now();
        let expires_at = now + Duration::days(REFRESH_TOKEN_EXPIRY_DAYS);
        
        Self {
            id: Uuid::new_v4(),
            user_id,
            token_hash,
            created_at: now,
            expires_at,
            is_revoked: false,
            token_family,
            device_fingerprint,
            previous_token_id,
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
    
    /// Token type (always "Bearer")
    pub token_type: String,
    
    /// Token issue timestamp
    pub issued_at: i64,
    
    /// Token family ID for rotation tracking
    pub token_family: Option<String>,
    
    /// Device fingerprint if provided
    pub device_fingerprint: Option<String>,
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
            token_type: "Bearer".to_string(),
            issued_at: Utc::now().timestamp(),
            token_family: None,
            device_fingerprint: None,
        }
    }
    
    /// Creates a new token pair with metadata
    ///
    /// # Arguments
    ///
    /// * `access_token` - The JWT access token
    /// * `refresh_token` - The JWT refresh token
    /// * `token_family` - Token family ID
    /// * `device_fingerprint` - Device fingerprint
    ///
    /// # Returns
    ///
    /// A new `TokenPair` instance with metadata
    pub fn new_with_metadata(
        access_token: String,
        refresh_token: String,
        token_family: Option<String>,
        device_fingerprint: Option<String>,
    ) -> Self {
        Self {
            access_token,
            refresh_token,
            access_expires_in: ACCESS_TOKEN_EXPIRY_MINUTES * 60,
            refresh_expires_in: REFRESH_TOKEN_EXPIRY_DAYS * 24 * 60 * 60,
            token_type: "Bearer".to_string(),
            issued_at: Utc::now().timestamp(),
            token_family,
            device_fingerprint,
        }
    }
}