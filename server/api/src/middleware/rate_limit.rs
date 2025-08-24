//! Rate limiting middleware for API endpoints
//!
//! This module provides rate limiting functionality to prevent API abuse
//! and brute force attacks. It uses Redis for distributed rate limiting
//! and supports different limits for different actions.

use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::{ErrorInternalServerError, ErrorTooManyRequests},
    http::StatusCode,
    Error, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use redis::{AsyncCommands, Client};
use serde_json::json;
use std::{
    collections::HashMap,
    future::{ready, Ready},
    rc::Rc,
    sync::Arc,
};

use crate::dto::error::{ErrorResponse, ErrorResponseExt};

/// Rate limit configuration for different actions
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// SMS sending: 3 times per phone number per hour
    pub sms_per_phone_per_hour: u32,
    /// Verification code validation: 3 attempts per phone number per code
    pub verification_attempts_per_code: u32,
    /// API calls: 60 requests per IP per minute
    pub api_calls_per_ip_per_minute: u32,
    /// Lock duration for phone numbers after max verification failures (in seconds)
    pub phone_lock_duration_seconds: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            sms_per_phone_per_hour: 3,
            verification_attempts_per_code: 3,
            api_calls_per_ip_per_minute: 60,
            phone_lock_duration_seconds: 1800, // 30 minutes
        }
    }
}

/// Rate limiter middleware factory
pub struct RateLimiter {
    redis_client: Arc<Client>,
    config: RateLimitConfig,
}

impl RateLimiter {
    /// Create a new rate limiter with Redis client
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = Client::open(redis_url)?;
        Ok(Self {
            redis_client: Arc::new(client),
            config: RateLimitConfig::default(),
        })
    }

    /// Create a new rate limiter with custom configuration
    pub fn with_config(redis_url: &str, config: RateLimitConfig) -> Result<Self, redis::RedisError> {
        let client = Client::open(redis_url)?;
        Ok(Self {
            redis_client: Arc::new(client),
            config,
        })
    }

    /// Check rate limit for a specific identifier and action
    async fn check_rate_limit(
        &self,
        identifier: &str,
        action: &str,
        limit: u32,
        window_seconds: u64,
    ) -> Result<RateLimitStatus, redis::RedisError> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let key = format!("rate_limit:{}:{}", action, identifier);

        // Get current count
        let count: Option<u32> = conn.get(&key).await?;

        match count {
            Some(current) if current >= limit => {
                // Rate limit exceeded
                let ttl: i64 = conn.ttl(&key).await?;
                Ok(RateLimitStatus::Exceeded {
                    retry_after_seconds: ttl.max(0) as u64,
                })
            }
            Some(current) => {
                // Increment counter
                let new_count: u32 = conn.incr(&key, 1).await?;
                Ok(RateLimitStatus::Ok {
                    remaining: limit.saturating_sub(new_count),
                })
            }
            None => {
                // First request, set counter with expiry
                conn.set_ex(&key, 1u32, window_seconds).await?;
                Ok(RateLimitStatus::Ok {
                    remaining: limit - 1,
                })
            }
        }
    }

    /// Check if a phone number is temporarily locked
    async fn is_phone_locked(&self, phone: &str) -> Result<bool, redis::RedisError> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let key = format!("phone_lock:{}", phone);
        let locked: bool = conn.exists(&key).await?;
        Ok(locked)
    }

    /// Lock a phone number temporarily
    async fn lock_phone(&self, phone: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let key = format!("phone_lock:{}", phone);
        conn.set_ex(&key, "locked", self.config.phone_lock_duration_seconds).await?;
        Ok(())
    }
}

/// Rate limit status
#[derive(Debug)]
enum RateLimitStatus {
    Ok { remaining: u32 },
    Exceeded { retry_after_seconds: u64 },
}

/// Middleware implementation for rate limiting
impl<S, B> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimiterMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimiterMiddleware {
            service: Rc::new(service),
            redis_client: self.redis_client.clone(),
            config: self.config.clone(),
        }))
    }
}

/// Rate limiter middleware service
pub struct RateLimiterMiddleware<S> {
    service: Rc<S>,
    redis_client: Arc<Client>,
    config: RateLimitConfig,
}

impl<S, B> Service<ServiceRequest> for RateLimiterMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let redis_client = self.redis_client.clone();
        let config = self.config.clone();

        Box::pin(async move {
            let path = req.path();

            // Determine rate limit type based on endpoint
            let rate_limit_result = if path.contains("/auth/send-code") {
                // SMS rate limiting per phone number
                if let Ok(phone) = extract_phone_from_request(&req).await {
                    check_sms_rate_limit(&redis_client, &phone, &config).await
                } else {
                    Ok(())
                }
            } else if path.contains("/auth/verify-code") {
                // Verification attempts rate limiting
                if let Ok(phone) = extract_phone_from_request(&req).await {
                    check_verification_rate_limit(&redis_client, &phone, &config).await
                } else {
                    Ok(())
                }
            } else {
                // General API rate limiting per IP
                let ip = get_client_ip(&req);
                check_api_rate_limit(&redis_client, &ip, &config).await
            };

            if let Err(error_response) = rate_limit_result {
                // Rate limit exceeded, return 429 error
                return Err(ErrorTooManyRequests(serde_json::json!({
                    "error": error_response.error,
                    "message": error_response.message,
                    "details": error_response.details,
                    "timestamp": error_response.timestamp
                })));
            }

            // Rate limit passed, continue with request
            service.call(req).await
        })
    }
}

/// Check SMS rate limit for a phone number
async fn check_sms_rate_limit(
    client: &Arc<Client>,
    phone: &str,
    config: &RateLimitConfig,
) -> Result<(), ErrorResponse> {
    let mut conn = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| {
            log::error!("Redis connection error: {:?}", e);
            ErrorResponse::new(
                "rate_limit_error".to_string(),
                "Unable to check rate limit".to_string(),
            )
        })?;

    // Check if phone is locked
    let lock_key = format!("phone_lock:{}", phone);
    let is_locked: bool = conn.exists(&lock_key).await.map_err(|e| {
        log::error!("Redis error checking phone lock: {:?}", e);
        ErrorResponse::new(
            "rate_limit_error".to_string(),
            "Unable to check rate limit".to_string(),
        )
    })?;

    if is_locked {
        let ttl: i64 = conn.ttl(&lock_key).await.unwrap_or(0);
        let minutes = (ttl.max(0) / 60) + 1;

        return Err(ErrorResponse::new(
            "phone_locked".to_string(),
            format!("Too many requests. Please try again in {} minutes | 请求过于频繁，请在 {} 分钟后重试", minutes, minutes),
        ).with_details(HashMap::from([
            ("retry_after_seconds".to_string(), json!(ttl.max(0))),
        ])));
    }

    // Check SMS rate limit
    let key = format!("sms_limit:{}", phone);
    let count: Option<u32> = conn.get(&key).await.map_err(|e| {
        log::error!("Redis error getting SMS count: {:?}", e);
        ErrorResponse::new(
            "rate_limit_error".to_string(),
            "Unable to check rate limit".to_string(),
        )
    })?;

    match count {
        Some(current) if current >= config.sms_per_phone_per_hour => {
            let ttl: i64 = conn.ttl(&key).await.unwrap_or(0);
            let minutes = (ttl.max(0) / 60) + 1;

            Err(ErrorResponse::new(
                "sms_rate_limit_exceeded".to_string(),
                format!("Too many SMS requests. Please try again in {} minutes | 短信请求过于频繁，请在 {} 分钟后重试", minutes, minutes),
            ).with_details(HashMap::from([
                ("retry_after_seconds".to_string(), json!(ttl.max(0))),
                ("limit".to_string(), json!(config.sms_per_phone_per_hour)),
                ("window".to_string(), json!("1 hour")),
            ])))
        }
        Some(_) | None => {
            // Increment or set counter
            let _: u32 = conn.incr(&key, 1).await.map_err(|e| {
                log::error!("Redis error incrementing SMS count: {:?}", e);
                ErrorResponse::new(
                    "rate_limit_error".to_string(),
                    "Unable to update rate limit".to_string(),
                )
            })?;

            // Set expiry on first request
            if count.is_none() {
                conn.expire(&key, 3600).await.map_err(|e| {
                    log::error!("Redis error setting expiry: {:?}", e);
                    ErrorResponse::new(
                        "rate_limit_error".to_string(),
                        "Unable to update rate limit".to_string(),
                    )
                })?;
            }

            Ok(())
        }
    }
}

/// Check verification rate limit for a phone number
async fn check_verification_rate_limit(
    client: &Arc<Client>,
    phone: &str,
    config: &RateLimitConfig,
) -> Result<(), ErrorResponse> {
    let mut conn = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| {
            log::error!("Redis connection error: {:?}", e);
            ErrorResponse::new(
                "rate_limit_error".to_string(),
                "Unable to check rate limit".to_string(),
            )
        })?;

    // Check if phone is locked
    let lock_key = format!("phone_lock:{}", phone);
    let is_locked: bool = conn.exists(&lock_key).await.map_err(|e| {
        log::error!("Redis error checking phone lock: {:?}", e);
        ErrorResponse::new(
            "rate_limit_error".to_string(),
            "Unable to check rate limit".to_string(),
        )
    })?;

    if is_locked {
        let ttl: i64 = conn.ttl(&lock_key).await.unwrap_or(0);
        let minutes = (ttl.max(0) / 60) + 1;

        return Err(ErrorResponse::new(
            "phone_locked".to_string(),
            format!("Account temporarily locked. Please try again in {} minutes | 账户暂时锁定，请在 {} 分钟后重试", minutes, minutes),
        ).with_details(HashMap::from([
            ("retry_after_seconds".to_string(), json!(ttl.max(0))),
        ])));
    }

    // Check verification attempts
    let key = format!("verify_attempts:{}", phone);
    let count: Option<u32> = conn.get(&key).await.map_err(|e| {
        log::error!("Redis error getting verification count: {:?}", e);
        ErrorResponse::new(
            "rate_limit_error".to_string(),
            "Unable to check rate limit".to_string(),
        )
    })?;

    match count {
        Some(current) if current >= config.verification_attempts_per_code => {
            // Lock the phone number
            conn.set_ex(&lock_key, "locked", config.phone_lock_duration_seconds)
                .await
                .map_err(|e| {
                    log::error!("Redis error locking phone: {:?}", e);
                    ErrorResponse::new(
                        "rate_limit_error".to_string(),
                        "Unable to update rate limit".to_string(),
                    )
                })?;

            // Clear the attempts counter
            let _: u32 = conn.del(&key).await.unwrap_or(0);

            Err(ErrorResponse::new(
                "max_attempts_exceeded".to_string(),
                "Maximum verification attempts exceeded. Account locked for 30 minutes | 验证尝试次数超限，账户锁定30分钟".to_string(),
            ).with_details(HashMap::from([
                ("lock_duration_seconds".to_string(), json!(config.phone_lock_duration_seconds)),
            ])))
        }
        Some(_) | None => {
            // Increment or set counter
            let _: u32 = conn.incr(&key, 1).await.map_err(|e| {
                log::error!("Redis error incrementing verification count: {:?}", e);
                ErrorResponse::new(
                    "rate_limit_error".to_string(),
                    "Unable to update rate limit".to_string(),
                )
            })?;

            // Set expiry on first request (5 minutes for verification attempts)
            if count.is_none() {
                conn.expire(&key, 300).await.map_err(|e| {
                    log::error!("Redis error setting expiry: {:?}", e);
                    ErrorResponse::new(
                        "rate_limit_error".to_string(),
                        "Unable to update rate limit".to_string(),
                    )
                })?;
            }

            Ok(())
        }
    }
}

/// Check general API rate limit per IP
async fn check_api_rate_limit(
    client: &Arc<Client>,
    ip: &str,
    config: &RateLimitConfig,
) -> Result<(), ErrorResponse> {
    let mut conn = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| {
            log::error!("Redis connection error: {:?}", e);
            ErrorResponse::new(
                "rate_limit_error".to_string(),
                "Unable to check rate limit".to_string(),
            )
        })?;

    let key = format!("api_limit:{}", ip);
    let count: Option<u32> = conn.get(&key).await.map_err(|e| {
        log::error!("Redis error getting API count: {:?}", e);
        ErrorResponse::new(
            "rate_limit_error".to_string(),
            "Unable to check rate limit".to_string(),
        )
    })?;

    match count {
        Some(current) if current >= config.api_calls_per_ip_per_minute => {
            let ttl: i64 = conn.ttl(&key).await.unwrap_or(0);

            Err(ErrorResponse::new(
                "api_rate_limit_exceeded".to_string(),
                "Too many requests. Please slow down | 请求过多，请放慢速度".to_string(),
            ).with_details(HashMap::from([
                ("retry_after_seconds".to_string(), json!(ttl.max(0))),
                ("limit".to_string(), json!(config.api_calls_per_ip_per_minute)),
                ("window".to_string(), json!("1 minute")),
            ])))
        }
        Some(_) | None => {
            // Increment or set counter
            let _: u32 = conn.incr(&key, 1).await.map_err(|e| {
                log::error!("Redis error incrementing API count: {:?}", e);
                ErrorResponse::new(
                    "rate_limit_error".to_string(),
                    "Unable to update rate limit".to_string(),
                )
            })?;

            // Set expiry on first request (1 minute for API calls)
            if count.is_none() {
                conn.expire(&key, 60).await.map_err(|e| {
                    log::error!("Redis error setting expiry: {:?}", e);
                    ErrorResponse::new(
                        "rate_limit_error".to_string(),
                        "Unable to update rate limit".to_string(),
                    )
                })?;
            }

            Ok(())
        }
    }
}

/// Extract phone number from request (placeholder - implement based on your request structure)
async fn extract_phone_from_request(_req: &ServiceRequest) -> Result<String, Error> {
    // This is a placeholder. In a real implementation, you would:
    // 1. Parse the request body or query parameters
    // 2. Extract the phone number field
    // For now, we'll return a placeholder error
    Err(ErrorInternalServerError("Phone extraction not implemented"))
}

/// Get client IP address from request
fn get_client_ip(req: &ServiceRequest) -> String {
    // Try to get IP from X-Forwarded-For header (for reverse proxy scenarios)
    if let Some(forwarded_for) = req.headers().get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded_for.to_str() {
            // Take the first IP from the comma-separated list
            if let Some(ip) = forwarded_str.split(',').next() {
                return ip.trim().to_string();
            }
        }
    }

    // Try to get IP from X-Real-IP header
    if let Some(real_ip) = req.headers().get("X-Real-IP") {
        if let Ok(ip_str) = real_ip.to_str() {
            return ip_str.to_string();
        }
    }

    // Fall back to connection info
    req.connection_info()
        .peer_addr()
        .unwrap_or("unknown")
        .to_string()
}
