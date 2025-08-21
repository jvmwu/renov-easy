//! OTP encryption service module for secure storage of verification codes

pub mod encrypted_cache_trait;
pub mod key_manager;
pub mod otp_encryption;
pub mod verification_adapter;

// Re-export main types
pub use encrypted_cache_trait::{EncryptedCacheServiceTrait, StorageBackend};
pub use key_manager::{KeyManager, KeyRotationConfig};
pub use otp_encryption::{
    AesGcmOtpEncryption, EncryptedOtp, OtpEncryption, OtpEncryptionConfig,
};
pub use verification_adapter::EncryptedVerificationAdapter;