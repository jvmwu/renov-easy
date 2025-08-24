//! Rate limiting configuration module

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Rate limiting configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// SMS rate limits
    pub sms: SmsRateLimits,

    /// API rate limits
    pub api: ApiRateLimits,

    /// Authentication rate limits
    pub auth: AuthRateLimits,

    /// Custom endpoint limits
    #[serde(default)]
    pub custom_limits: HashMap<String, EndpointLimit>,
}

/// SMS-specific rate limits
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SmsRateLimits {
    /// Max SMS per phone number per hour
    pub per_phone_per_hour: u32,

    /// Max SMS per phone number per day
    pub per_phone_per_day: u32,

    /// Max verification attempts per code
    pub verification_attempts_per_code: u32,

    /// Phone number lock duration in seconds after exceeding limits
    pub phone_lock_duration: u64,

    /// Cooldown period between SMS sends in seconds
    #[serde(default = "default_sms_cooldown")]
    pub cooldown_seconds: u64,
}

impl Default for SmsRateLimits {
    fn default() -> Self {
        Self {
            per_phone_per_hour: 3,
            per_phone_per_day: 10,
            verification_attempts_per_code: 3,
            phone_lock_duration: 3600,  // 1 hour
            cooldown_seconds: default_sms_cooldown(),
        }
    }
}

/// API-specific rate limits
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiRateLimits {
    /// Max requests per IP per minute
    pub per_ip_per_minute: u32,

    /// Max requests per IP per hour
    pub per_ip_per_hour: u32,

    /// Max requests per authenticated user per minute
    pub per_user_per_minute: u32,

    /// Max requests per authenticated user per hour
    pub per_user_per_hour: u32,

    /// Burst limit (max requests in a short burst)
    #[serde(default = "default_burst_limit")]
    pub burst_limit: u32,

    /// Burst window in seconds
    #[serde(default = "default_burst_window")]
    pub burst_window: u64,
}

impl Default for ApiRateLimits {
    fn default() -> Self {
        Self {
            per_ip_per_minute: 60,
            per_ip_per_hour: 1000,
            per_user_per_minute: 100,
            per_user_per_hour: 2000,
            burst_limit: default_burst_limit(),
            burst_window: default_burst_window(),
        }
    }
}

/// Authentication-specific rate limits
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthRateLimits {
    /// Max login attempts per IP per hour
    pub login_per_ip_per_hour: u32,

    /// Max login attempts per username per hour
    pub login_per_user_per_hour: u32,

    /// Max password reset requests per email per day
    pub password_reset_per_day: u32,

    /// Account lock duration after failed attempts in seconds
    pub account_lock_duration: u64,

    /// Number of failed attempts before locking
    #[serde(default = "default_failed_attempts_threshold")]
    pub failed_attempts_threshold: u32,
}

impl Default for AuthRateLimits {
    fn default() -> Self {
        Self {
            login_per_ip_per_hour: 10,
            login_per_user_per_hour: 5,
            password_reset_per_day: 3,
            account_lock_duration: 1800,  // 30 minutes
            failed_attempts_threshold: default_failed_attempts_threshold(),
        }
    }
}

/// Custom endpoint rate limit
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EndpointLimit {
    /// Endpoint path pattern (e.g., "/api/v1/orders/*")
    pub path_pattern: String,

    /// Max requests per minute
    pub per_minute: u32,

    /// Max requests per hour
    pub per_hour: u32,

    /// Apply to authenticated users only
    #[serde(default)]
    pub authenticated_only: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            sms: SmsRateLimits::default(),
            api: ApiRateLimits::default(),
            auth: AuthRateLimits::default(),
            custom_limits: HashMap::new(),
        }
    }
}

impl RateLimitConfig {
    /// Get max requests (backward compatibility - returns SMS per hour)
    pub fn max_requests(&self) -> u32 {
        self.sms.per_phone_per_hour
    }

    /// Get window seconds (backward compatibility - returns 3600 for 1 hour)
    pub fn window_seconds(&self) -> u64 {
        3600  // 1 hour window for SMS rate limiting
    }

    /// Add a custom endpoint limit
    pub fn add_custom_limit(mut self, name: impl Into<String>, limit: EndpointLimit) -> Self {
        self.custom_limits.insert(name.into(), limit);
        self
    }

    /// Create a development configuration (more lenient limits)
    pub fn development() -> Self {
        Self {
            enabled: true,
            sms: SmsRateLimits {
                per_phone_per_hour: 10,
                per_phone_per_day: 50,
                ..Default::default()
            },
            api: ApiRateLimits {
                per_ip_per_minute: 300,
                per_ip_per_hour: 10000,
                ..Default::default()
            },
            auth: AuthRateLimits {
                login_per_ip_per_hour: 100,
                login_per_user_per_hour: 50,
                ..Default::default()
            },
            custom_limits: HashMap::new(),
        }
    }

    /// Create a production configuration (stricter limits)
    pub fn production() -> Self {
        Self::default()
    }
}

fn default_enabled() -> bool {
    true
}

fn default_sms_cooldown() -> u64 {
    60  // 1 minute
}

fn default_burst_limit() -> u32 {
    10
}

fn default_burst_window() -> u64 {
    1  // 1 second
}

fn default_failed_attempts_threshold() -> u32 {
    5
}
