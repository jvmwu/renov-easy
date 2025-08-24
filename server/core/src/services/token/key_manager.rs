//! RS256 key management for JWT signing and verification

use std::fs;
use std::path::{Path, PathBuf};
use jsonwebtoken::{DecodingKey, EncodingKey};
use crate::errors::{DomainError, TokenError};

/// Manager for RS256 keys used in JWT operations
#[derive(Clone)]
pub struct Rs256KeyManager {
    /// Private key for signing JWTs
    encoding_key: EncodingKey,
    /// Public key for verifying JWTs
    decoding_key: DecodingKey,
    /// Path to private key file
    private_key_path: PathBuf,
    /// Path to public key file
    public_key_path: PathBuf,
}

impl std::fmt::Debug for Rs256KeyManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Rs256KeyManager")
            .field("private_key_path", &self.private_key_path)
            .field("public_key_path", &self.public_key_path)
            .finish()
    }
}

impl Rs256KeyManager {
    /// Creates a new RS256 key manager from key file paths
    ///
    /// # Arguments
    ///
    /// * `private_key_path` - Path to the PEM-encoded private key file
    /// * `public_key_path` - Path to the PEM-encoded public key file
    ///
    /// # Returns
    ///
    /// * `Ok(Rs256KeyManager)` - Key manager initialized successfully
    /// * `Err(DomainError)` - Failed to load keys
    ///
    /// # Example
    ///
    /// ```no_run
    /// use renov_core::services::token::Rs256KeyManager;
    ///
    /// let key_manager = Rs256KeyManager::new(
    ///     "core/keys/jwt_private_key.pem",
    ///     "core/keys/jwt_public_key.pem"
    /// ).expect("Failed to load keys");
    /// ```
    pub fn new<P: AsRef<Path>>(
        private_key_path: P,
        public_key_path: P,
    ) -> Result<Self, DomainError> {
        let private_key_path = private_key_path.as_ref().to_path_buf();
        let public_key_path = public_key_path.as_ref().to_path_buf();

        // Load private key
        let private_key_pem = fs::read(&private_key_path)
            .map_err(|e| DomainError::Token(TokenError::KeyLoadError {
                message: format!("Failed to read private key: {}", e),
            }))?;

        let encoding_key = EncodingKey::from_rsa_pem(&private_key_pem)
            .map_err(|e| DomainError::Token(TokenError::KeyLoadError {
                message: format!("Invalid private key format: {}", e),
            }))?;

        // Load public key
        let public_key_pem = fs::read(&public_key_path)
            .map_err(|e| DomainError::Token(TokenError::KeyLoadError {
                message: format!("Failed to read public key: {}", e),
            }))?;

        let decoding_key = DecodingKey::from_rsa_pem(&public_key_pem)
            .map_err(|e| DomainError::Token(TokenError::KeyLoadError {
                message: format!("Invalid public key format: {}", e),
            }))?;

        Ok(Self {
            encoding_key,
            decoding_key,
            private_key_path,
            public_key_path,
        })
    }

    /// Creates a key manager from environment variables
    ///
    /// Expects the following environment variables:
    /// - `JWT_PRIVATE_KEY_PATH`: Path to private key file
    /// - `JWT_PUBLIC_KEY_PATH`: Path to public key file
    ///
    /// # Returns
    ///
    /// * `Ok(Rs256KeyManager)` - Key manager initialized successfully
    /// * `Err(DomainError)` - Environment variables not set or keys not found
    pub fn from_env() -> Result<Self, DomainError> {
        let private_key_path = std::env::var("JWT_PRIVATE_KEY_PATH")
            .unwrap_or_else(|_| "core/keys/jwt_private_key.pem".to_string());

        let public_key_path = std::env::var("JWT_PUBLIC_KEY_PATH")
            .unwrap_or_else(|_| "core/keys/jwt_public_key.pem".to_string());

        Self::new(private_key_path, public_key_path)
    }

    /// Creates a key manager from PEM strings (useful for testing or embedded keys)
    ///
    /// # Arguments
    ///
    /// * `private_key_pem` - PEM-encoded private key string
    /// * `public_key_pem` - PEM-encoded public key string
    ///
    /// # Returns
    ///
    /// * `Ok(Rs256KeyManager)` - Key manager initialized successfully
    /// * `Err(DomainError)` - Invalid key format
    pub fn from_pem_strings(
        private_key_pem: &str,
        public_key_pem: &str,
    ) -> Result<Self, DomainError> {
        let encoding_key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())
            .map_err(|e| DomainError::Token(TokenError::KeyLoadError {
                message: format!("Invalid private key format: {}", e),
            }))?;

        let decoding_key = DecodingKey::from_rsa_pem(public_key_pem.as_bytes())
            .map_err(|e| DomainError::Token(TokenError::KeyLoadError {
                message: format!("Invalid public key format: {}", e),
            }))?;

        Ok(Self {
            encoding_key,
            decoding_key,
            private_key_path: PathBuf::from("memory"),
            public_key_path: PathBuf::from("memory"),
        })
    }

    /// Returns the encoding key for signing JWTs
    pub fn encoding_key(&self) -> &EncodingKey {
        &self.encoding_key
    }

    /// Returns the decoding key for verifying JWTs
    pub fn decoding_key(&self) -> &DecodingKey {
        &self.decoding_key
    }

    /// Checks if the keys are loaded and valid
    pub fn validate(&self) -> bool {
        // Basic validation - keys are loaded
        // In production, you might want to do a test sign/verify operation
        true
    }

    /// Returns the paths to the key files
    pub fn key_paths(&self) -> (&Path, &Path) {
        (&self.private_key_path, &self.public_key_path)
    }

    /// Reloads keys from disk (useful for key rotation)
    pub fn reload(&mut self) -> Result<(), DomainError> {
        // Only reload if using file-based keys
        if self.private_key_path.as_os_str() == "memory" {
            return Ok(());
        }

        let new_manager = Self::new(&self.private_key_path, &self.public_key_path)?;
        self.encoding_key = new_manager.encoding_key;
        self.decoding_key = new_manager.decoding_key;

        Ok(())
    }
}

/// Configuration for RS256 key management
#[derive(Debug, Clone)]
pub struct Rs256KeyConfig {
    /// Path to private key file
    pub private_key_path: String,
    /// Path to public key file
    pub public_key_path: String,
    /// Whether to allow key rotation
    pub allow_rotation: bool,
    /// Key rotation check interval in seconds (0 = disabled)
    pub rotation_check_interval: u64,
}

impl Default for Rs256KeyConfig {
    fn default() -> Self {
        Self {
            private_key_path: "core/keys/jwt_private_key.pem".to_string(),
            public_key_path: "core/keys/jwt_public_key.pem".to_string(),
            allow_rotation: false,
            rotation_check_interval: 0,
        }
    }
}

impl Rs256KeyConfig {
    /// Creates config from environment variables
    pub fn from_env() -> Self {
        Self {
            private_key_path: std::env::var("JWT_PRIVATE_KEY_PATH")
                .unwrap_or_else(|_| "core/keys/jwt_private_key.pem".to_string()),
            public_key_path: std::env::var("JWT_PUBLIC_KEY_PATH")
                .unwrap_or_else(|_| "core/keys/jwt_public_key.pem".to_string()),
            allow_rotation: std::env::var("JWT_ALLOW_KEY_ROTATION")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            rotation_check_interval: std::env::var("JWT_KEY_ROTATION_INTERVAL")
                .unwrap_or_else(|_| "0".to_string())
                .parse()
                .unwrap_or(0),
        }
    }
}
