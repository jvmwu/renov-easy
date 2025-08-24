//! Cache module for Redis-based caching
//! 
//! This module provides Redis caching functionality for the RenovEasy application,
//! including connection pooling, retry logic, and common cache operations.

pub mod otp_storage;
pub mod redis_client;
pub mod verification_cache;

#[cfg(test)]
mod tests;

pub use otp_storage::{OtpRedisStorage, OtpStorageConfig, OtpMetadata};
pub use redis_client::RedisClient;
pub use verification_cache::VerificationCache;

// Re-export commonly used types
pub use re_shared::config::cache::CacheConfig;