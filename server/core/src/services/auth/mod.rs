//! Authentication service module
//!
//! This module provides a complete authentication system including:
//! - Phone number verification via SMS
//! - User registration and login
//! - Token generation and refresh
//! - User type selection
//! - Rate limiting

mod config;
mod phone_utils;
mod rate_limiter;
mod service;

#[cfg(test)]
mod tests;

pub use config::AuthServiceConfig;
pub use rate_limiter::RateLimiterTrait;
pub use service::AuthService;

// Export selected phone utilities for public use
pub use phone_utils::{
    validate_chinese_phone,
    validate_australian_phone,
    validate_phone_with_country,
    normalize_to_e164,
    mask_phone,
    CountryCode,
};
