//! Verification service module for SMS-based authentication
//!
//! This module provides a complete verification code workflow including:
//! - SMS code generation and sending
//! - Code verification with attempt tracking
//! - Rate limiting and cooldown periods
//! - Integration with SMS and cache services
//! - Enhanced security with account locking and brute force protection

mod config;
mod enhanced_verification;
mod service;
mod traits;
mod types;

#[cfg(test)]
mod tests;

pub use config::VerificationServiceConfig;
pub use enhanced_verification::{
    AccountLockInfo, EnhancedVerificationService, LockReason, VerificationStats,
};
pub use service::VerificationService;
pub use traits::{SmsServiceTrait, CacheServiceTrait};
pub use types::{SendCodeResult, VerifyCodeResult};