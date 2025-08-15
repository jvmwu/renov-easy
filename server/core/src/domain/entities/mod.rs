//! Domain entities representing core business objects.

pub mod token;
pub mod user;
pub mod verification_code;

// Placeholder for future entity modules
// pub mod worker;
// pub mod order;

// Re-export commonly used types
pub use token::{
    Claims, RefreshToken, TokenPair,
    ACCESS_TOKEN_EXPIRY_MINUTES, REFRESH_TOKEN_EXPIRY_DAYS,
    JWT_ISSUER, JWT_AUDIENCE
};
pub use user::{User, UserType};
pub use verification_code::{VerificationCode, MAX_ATTEMPTS, CODE_LENGTH, DEFAULT_EXPIRATION_MINUTES};