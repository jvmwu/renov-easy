//! Main token service implementation

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use rand::Rng;
use chrono::TimeZone;

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
    /// * `phone_hash` - Hashed phone number
    /// * `device_fingerprint` - Device fingerprint for tracking
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
        phone_hash: Option<String>,
        device_fingerprint: Option<String>,
    ) -> Result<TokenPair, DomainError> {
        // Generate token family ID for new token chains
        let token_family = Some(Uuid::new_v4().to_string());
        
        // Generate access token
        let access_token = self.generate_access_token(
            user_id,
            user_type.clone(),
            is_verified,
            phone_hash,
            device_fingerprint.clone(),
        )?;
        
        // Generate refresh token with family tracking
        let refresh_token = self.generate_refresh_token(
            user_id,
            token_family.clone(),
            device_fingerprint.clone(),
            None,
        ).await?;
        
        Ok(TokenPair::new_with_metadata(
            access_token,
            refresh_token,
            token_family,
            device_fingerprint,
        ))
    }

    /// Generates an access token
    fn generate_access_token(
        &self,
        user_id: Uuid,
        user_type: Option<UserType>,
        is_verified: bool,
        phone_hash: Option<String>,
        device_fingerprint: Option<String>,
    ) -> Result<String, DomainError> {
        let user_type_str = user_type.map(|ut| match ut {
            UserType::Customer => "customer".to_string(),
            UserType::Worker => "worker".to_string(),
        });
        let claims = Claims::new_access_token(
            user_id,
            user_type_str,
            is_verified,
            phone_hash,
            device_fingerprint,
        );
        self.encode_jwt(&claims)
    }

    /// Generates a refresh token and stores it
    async fn generate_refresh_token(
        &self,
        user_id: Uuid,
        token_family: Option<String>,
        device_fingerprint: Option<String>,
        previous_token_id: Option<Uuid>,
    ) -> Result<String, DomainError> {
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
        let refresh_token = RefreshToken::new_with_metadata(
            user_id,
            token_hash,
            token_family,
            device_fingerprint,
            previous_token_id,
        );
        
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
    pub async fn verify_access_token(&self, token: &str) -> Result<Claims, DomainError> {
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
        
        // Check if token is blacklisted
        if self.repository.is_token_blacklisted(&token_data.claims.jti).await
            .unwrap_or(false) {
            return Err(DomainError::Token(TokenError::TokenRevoked));
        }
        
        Ok(token_data.claims)
    }
    
    /// Verifies an access token synchronously (backward compatibility)
    ///
    /// # Arguments
    ///
    /// * `token` - The JWT access token to verify
    ///
    /// # Returns
    ///
    /// * `Ok(Claims)` - The decoded claims if valid
    /// * `Err(TokenError)` - Token is invalid, expired, or malformed
    pub fn verify_access_token_sync(&self, token: &str) -> Result<Claims, DomainError> {
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
        
        // Note: Cannot check blacklist synchronously
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

    /// Refreshes tokens using a refresh token (with rotation)
    ///
    /// # Arguments
    ///
    /// * `refresh_token` - The refresh token
    /// * `user_type` - Updated user type (if changed)
    /// * `is_verified` - Updated verification status
    /// * `phone_hash` - Updated phone hash
    /// * `device_fingerprint` - Device fingerprint for validation
    ///
    /// # Returns
    ///
    /// * `Ok(TokenPair)` - New token pair (rotated)
    /// * `Err(TokenError)` - Refresh failed
    pub async fn refresh_tokens(
        &self,
        refresh_token: &str,
        user_type: Option<UserType>,
        is_verified: bool,
        phone_hash: Option<String>,
        device_fingerprint: Option<String>,
    ) -> Result<TokenPair, DomainError> {
        let token_hash = self.hash_token(refresh_token);
        
        // Find and verify the refresh token
        let old_token = self.repository
            .find_refresh_token(&token_hash)
            .await
            .map_err(|_| DomainError::Token(TokenError::InvalidTokenFormat))?
            .ok_or(DomainError::Token(TokenError::InvalidTokenFormat))?;
        
        // Check if token is expired
        if old_token.is_expired() {
            return Err(DomainError::Token(TokenError::TokenExpired));
        }
        
        // Check if token is revoked
        if old_token.is_revoked {
            // If token is revoked, it might be a reuse attack
            // Revoke entire token family for security
            if let Some(ref family) = old_token.token_family {
                let _ = self.repository.revoke_token_family(family).await;
            }
            return Err(DomainError::Token(TokenError::TokenRevoked));
        }
        
        // Validate device fingerprint if present
        if let (Some(ref token_fp), Some(ref provided_fp)) = 
            (&old_token.device_fingerprint, &device_fingerprint) {
            if token_fp != provided_fp {
                // Device mismatch - potential security issue
                // Revoke the token family
                if let Some(ref family) = old_token.token_family {
                    let _ = self.repository.revoke_token_family(family).await;
                }
                return Err(DomainError::Token(TokenError::InvalidTokenFormat));
            }
        }
        
        // Generate new access token
        let access_token = self.generate_access_token(
            old_token.user_id,
            user_type,
            is_verified,
            phone_hash,
            device_fingerprint.clone(),
        )?;
        
        // Rotate refresh token (generate new one, revoke old one)
        let new_refresh_token = self.generate_refresh_token(
            old_token.user_id,
            old_token.token_family.clone(),
            device_fingerprint.clone(),
            Some(old_token.id),
        ).await?;
        
        // Revoke the old refresh token
        let _ = self.repository.revoke_token(&token_hash).await;
        
        Ok(TokenPair::new_with_metadata(
            access_token,
            new_refresh_token,
            old_token.token_family,
            device_fingerprint,
        ))
    }
    
    /// Refreshes an access token only (backward compatibility)
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
        self.generate_access_token(user_id, user_type, is_verified, None, None)
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
    
    /// Blacklists an access token by its JWT ID
    ///
    /// # Arguments
    ///
    /// * `token` - The JWT access token to blacklist
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Token blacklisted successfully
    /// * `Err(TokenError)` - Blacklisting failed
    pub async fn blacklist_access_token(&self, token: &str) -> Result<(), DomainError> {
        // Decode the token to get the JTI and expiry
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map_err(|_| DomainError::Token(TokenError::InvalidTokenFormat))?;
        
        let expires_at = chrono::Utc.timestamp_opt(token_data.claims.exp, 0)
            .single()
            .ok_or(DomainError::Internal {
                message: "Invalid expiry timestamp".to_string(),
            })?;
        
        self.repository
            .blacklist_token(&token_data.claims.jti, expires_at)
            .await
            .map_err(|_| DomainError::Internal {
                message: "Failed to blacklist token".to_string(),
            })
    }
    
    /// Revokes all tokens for a specific device
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user's UUID
    /// * `device_fingerprint` - The device fingerprint
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` - Number of tokens revoked
    /// * `Err(TokenError)` - Revocation failed
    pub async fn revoke_device_tokens(
        &self,
        user_id: Uuid,
        device_fingerprint: &str,
    ) -> Result<usize, DomainError> {
        // Find all tokens for the user
        let tokens = self.repository
            .find_by_user_id(user_id)
            .await
            .map_err(|_| DomainError::Internal {
                message: "Failed to find user tokens".to_string(),
            })?;
        
        let mut revoked_count = 0;
        for token in tokens {
            if let Some(ref fp) = token.device_fingerprint {
                if fp == device_fingerprint && !token.is_revoked {
                    if self.repository.revoke_token(&token.token_hash).await.unwrap_or(false) {
                        revoked_count += 1;
                    }
                }
            }
        }
        
        Ok(revoked_count)
    }
    
    /// Cleans up expired tokens and blacklist entries
    ///
    /// # Returns
    ///
    /// * `Ok((tokens_deleted, blacklist_deleted))` - Cleanup counts
    /// * `Err(TokenError)` - Cleanup failed
    pub async fn cleanup_all(&self) -> Result<(usize, usize), DomainError> {
        let tokens_deleted = self.cleanup_expired_tokens().await?;
        let blacklist_deleted = self.repository
            .cleanup_blacklist()
            .await
            .unwrap_or(0);
        
        Ok((tokens_deleted, blacklist_deleted))
    }
}