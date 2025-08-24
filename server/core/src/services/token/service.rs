//! Main token service implementation

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use rand::Rng;

use crate::domain::entities::token::{Claims, RefreshToken, TokenPair};
use crate::domain::entities::user::UserType;
use crate::errors::{DomainError, TokenError};
use crate::repositories::TokenRepository;

use super::config::TokenServiceConfig;
use super::key_manager::Rs256KeyManager;

/// Service for managing JWT tokens and refresh tokens
pub struct TokenService<R: TokenRepository> {
    pub(crate) repository: R,
    config: TokenServiceConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
    /// Optional RS256 key manager for asymmetric signing
    rs256_key_manager: Option<Rs256KeyManager>,
}

impl<R: TokenRepository> TokenService<R> {
    /// Creates a new token service instance
    ///
    /// # Arguments
    ///
    /// * `repository` - Token repository for persistence
    /// * `config` - Token service configuration
    ///
    /// # Returns
    ///
    /// A new `TokenService` instance or error if key loading fails
    pub fn new(repository: R, config: TokenServiceConfig) -> Result<Self, DomainError> {
        // Load RS256 keys if configured
        let (encoding_key, decoding_key, rs256_key_manager) = 
            if config.algorithm == Algorithm::RS256 {
                // Load RS256 key manager
                let manager = config.load_key_manager()?
                    .ok_or_else(|| DomainError::Internal {
                        message: "RS256 algorithm requires key configuration".to_string(),
                    })?;
                
                let encoding_key = manager.encoding_key().clone();
                let decoding_key = manager.decoding_key().clone();
                
                (encoding_key, decoding_key, Some(manager))
            } else {
                // Use HS256 with symmetric key (backward compatibility)
                let encoding_key = EncodingKey::from_secret(config.jwt_secret.as_bytes());
                let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_bytes());
                (encoding_key, decoding_key, None)
            };
        
        let mut validation = Validation::new(config.algorithm);
        validation.set_issuer(&["renov-easy"]);
        validation.set_audience(&["renov-easy-api"]);
        validation.validate_exp = true;
        validation.validate_nbf = true;
        
        Ok(Self {
            repository,
            config,
            encoding_key,
            decoding_key,
            validation,
            rs256_key_manager,
        })
    }
    
    /// Creates a new token service with explicit RS256 key manager
    ///
    /// # Arguments
    ///
    /// * `repository` - Token repository for persistence
    /// * `config` - Token service configuration
    /// * `key_manager` - RS256 key manager for asymmetric signing
    ///
    /// # Returns
    ///
    /// A new `TokenService` instance
    pub fn with_rs256_keys(
        repository: R,
        mut config: TokenServiceConfig,
        key_manager: Rs256KeyManager,
    ) -> Self {
        // Ensure config uses RS256
        config.algorithm = Algorithm::RS256;
        
        let encoding_key = key_manager.encoding_key().clone();
        let decoding_key = key_manager.decoding_key().clone();
        
        let mut validation = Validation::new(config.algorithm);
        validation.set_issuer(&["renov-easy"]);
        validation.set_audience(&["renov-easy-api"]);
        validation.validate_exp = true;
        validation.validate_nbf = true;
        
        Self {
            repository,
            config,
            encoding_key,
            decoding_key,
            validation,
            rs256_key_manager: Some(key_manager),
        }
    }

    /// Generates a new token pair (access + refresh tokens) for a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user's UUID
    /// * `user_type` - The user's type (Customer or Worker)
    /// * `is_verified` - Whether the user is verified
    ///
    /// # Returns
    ///
    /// * `Ok(TokenPair)` - The generated token pair
    /// * `Err(TokenError)` - Token generation failed
    pub async fn generate_tokens(
        &self,
        user_id: Uuid,
        user_type: Option<UserType>,
        is_verified: bool,
    ) -> Result<TokenPair, DomainError> {
        // Generate access token
        let access_token = self.generate_access_token(user_id, user_type.clone(), is_verified)?;
        
        // Generate refresh token
        let refresh_token = self.generate_refresh_token(user_id).await?;
        
        Ok(TokenPair {
            access_token,
            refresh_token,
            access_expires_in: self.config.access_token_expiry_minutes * 60,
            refresh_expires_in: self.config.refresh_token_expiry_days * 24 * 60 * 60,
        })
    }

    /// Generates an access token
    fn generate_access_token(
        &self,
        user_id: Uuid,
        user_type: Option<UserType>,
        is_verified: bool,
    ) -> Result<String, DomainError> {
        let user_type_str = user_type.map(|ut| match ut {
            UserType::Customer => "customer".to_string(),
            UserType::Worker => "worker".to_string(),
        });
        let claims = Claims::new_access_token(user_id, user_type_str, is_verified);
        self.encode_jwt(&claims)
    }

    /// Generates a refresh token and stores it
    async fn generate_refresh_token(&self, user_id: Uuid) -> Result<String, DomainError> {
        // Generate a random token string
        let mut rng = rand::thread_rng();
        let token_string: String = (0..32)
            .map(|_| {
                let idx = rng.gen_range(0..62);
                match idx {
                    0..10 => (b'0' + idx) as char,
                    10..36 => (b'a' + idx - 10) as char,
                    36..62 => (b'A' + idx - 36) as char,
                    _ => unreachable!(),
                }
            })
            .collect();
        
        // Hash the token for storage
        let token_hash = self.hash_token(&token_string);
        let refresh_token = RefreshToken::new(user_id, token_hash);
        
        // Store the refresh token
        self.repository
            .save_refresh_token(refresh_token)
            .await
            .map_err(|_| DomainError::Token(TokenError::TokenGenerationFailed))?;
        
        Ok(token_string)
    }

    /// Encodes claims into a JWT
    pub(crate) fn encode_jwt(&self, claims: &Claims) -> Result<String, DomainError> {
        let header = Header::new(self.config.algorithm);
        encode(&header, claims, &self.encoding_key)
            .map_err(|_| DomainError::Token(TokenError::TokenGenerationFailed))
    }

    /// Verifies an access token and returns the claims
    ///
    /// # Arguments
    ///
    /// * `token` - The JWT access token to verify
    ///
    /// # Returns
    ///
    /// * `Ok(Claims)` - The decoded claims if valid
    /// * `Err(TokenError)` - Token is invalid, expired, or malformed
    pub fn verify_access_token(&self, token: &str) -> Result<Claims, DomainError> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| {
                if e.kind() == &jsonwebtoken::errors::ErrorKind::ExpiredSignature {
                    DomainError::Token(TokenError::TokenExpired)
                } else if e.kind() == &jsonwebtoken::errors::ErrorKind::ImmatureSignature {
                    DomainError::Token(TokenError::TokenNotYetValid)
                } else {
                    DomainError::Token(TokenError::InvalidTokenFormat)
                }
            })?;
        
        Ok(token_data.claims)
    }

    /// Verifies a refresh token and returns the user ID
    ///
    /// # Arguments
    ///
    /// * `token` - The refresh token to verify
    ///
    /// # Returns
    ///
    /// * `Ok(Uuid)` - The user ID if token is valid
    /// * `Err(TokenError)` - Token is invalid, expired, or revoked
    pub async fn verify_refresh_token(&self, token: &str) -> Result<Uuid, DomainError> {
        let token_hash = self.hash_token(token);
        
        let refresh_token = self.repository
            .find_refresh_token(&token_hash)
            .await
            .map_err(|_| DomainError::Token(TokenError::InvalidTokenFormat))?
            .ok_or(DomainError::Token(TokenError::InvalidTokenFormat))?;
        
        // Check if token is expired
        if refresh_token.is_expired() {
            return Err(DomainError::Token(TokenError::TokenExpired));
        }
        
        // Check if token is revoked
        if refresh_token.is_revoked {
            return Err(DomainError::Token(TokenError::TokenRevoked));
        }
        
        Ok(refresh_token.user_id)
    }

    /// Refreshes an access token using a refresh token
    ///
    /// # Arguments
    ///
    /// * `refresh_token` - The refresh token
    /// * `user_type` - Updated user type (if changed)
    /// * `is_verified` - Updated verification status
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - New access token
    /// * `Err(TokenError)` - Refresh failed
    pub async fn refresh_access_token(
        &self,
        refresh_token: &str,
        user_type: Option<UserType>,
        is_verified: bool,
    ) -> Result<String, DomainError> {
        // Verify the refresh token
        let user_id = self.verify_refresh_token(refresh_token).await?;
        
        // Generate new access token
        self.generate_access_token(user_id, user_type, is_verified)
    }

    /// Revokes all tokens for a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user's UUID
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Tokens revoked successfully
    /// * `Err(TokenError)` - Revocation failed
    pub async fn revoke_tokens(&self, user_id: Uuid) -> Result<(), DomainError> {
        self.repository
            .revoke_all_user_tokens(user_id)
            .await
            .map(|_| ())
            .map_err(|_| DomainError::Token(TokenError::TokenGenerationFailed))
    }

    /// Revokes a specific refresh token
    ///
    /// # Arguments
    ///
    /// * `token` - The refresh token to revoke
    ///
    /// # Returns
    ///
    /// * `Ok(bool)` - True if token was revoked, false if not found
    /// * `Err(TokenError)` - Revocation failed
    pub async fn revoke_refresh_token(&self, token: &str) -> Result<bool, DomainError> {
        let token_hash = self.hash_token(token);
        
        self.repository
            .revoke_token(&token_hash)
            .await
            .map_err(|_| DomainError::Token(TokenError::TokenGenerationFailed))
    }

    /// Removes expired tokens from storage
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` - Number of tokens cleaned up
    /// * `Err(TokenError)` - Cleanup failed
    pub async fn cleanup_expired_tokens(&self) -> Result<usize, DomainError> {
        self.repository
            .delete_expired_tokens()
            .await
            .map_err(|_| DomainError::Internal {
                message: "Failed to cleanup expired tokens".to_string(),
            })
    }

    /// Hashes a token for secure storage
    pub(crate) fn hash_token(&self, token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}