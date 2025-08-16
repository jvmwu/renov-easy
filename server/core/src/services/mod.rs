//! Business services containing domain logic and use cases.

pub mod auth_service;
pub mod token_service;
pub mod verification_service;

// Re-export commonly used types
pub use auth_service::{AuthService, AuthServiceConfig, RateLimiterTrait};
pub use token_service::{TokenService, TokenServiceConfig};
pub use verification_service::{
    VerificationService, VerificationServiceConfig, 
    SendCodeResult, VerifyCodeResult,
    SmsServiceTrait, CacheServiceTrait,
};

// Placeholder for future service modules
// pub mod order_service;
// pub mod user_service;
// pub mod worker_service;