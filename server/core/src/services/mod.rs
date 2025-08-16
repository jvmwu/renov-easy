//! Business services containing domain logic and use cases.

pub mod auth;
pub mod token;
pub mod verification;

// Re-export commonly used types
pub use auth::{AuthService, AuthServiceConfig, RateLimiterTrait};
pub use token::{TokenService, TokenServiceConfig};
pub use verification::{
    VerificationService, VerificationServiceConfig, 
    SendCodeResult, VerifyCodeResult,
    SmsServiceTrait, CacheServiceTrait,
};

// Placeholder for future service modules
// pub mod order_service;
// pub mod user_service;
// pub mod worker_service;