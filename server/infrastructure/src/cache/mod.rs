//! Cache module for Redis-based caching
//! 
//! This module provides Redis caching functionality for the RenovEasy application,
//! including connection pooling, retry logic, and common cache operations.

pub mod redis_client;
pub mod verification_cache;

pub use redis_client::RedisClient;
pub use verification_cache::VerificationCache;

// Re-export commonly used types
pub use crate::config::CacheConfig;