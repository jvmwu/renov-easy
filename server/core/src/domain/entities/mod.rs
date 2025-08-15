//! Domain entities representing core business objects.

pub mod user;
pub mod verification_code;

// Placeholder for future entity modules
// pub mod worker;
// pub mod order;

// Re-export commonly used types
pub use user::{User, UserType};
pub use verification_code::{VerificationCode, MAX_ATTEMPTS, CODE_LENGTH, DEFAULT_EXPIRATION_MINUTES};