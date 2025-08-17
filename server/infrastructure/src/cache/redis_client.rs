//! Redis cache client implementation
//! 
//! This module provides a Redis client with connection pooling, retry logic,
//! and basic cache operations for the RenovEasy infrastructure layer.
//! It supports operations like set with expiry, get, and delete for caching
//! verification codes, session data, and rate limiting counters.

use redis::{
    aio::MultiplexedConnection,
    AsyncCommands, Client, RedisError, RedisResult,
};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

use crate::config::CacheConfig;
use crate::InfrastructureError;

/// Redis cache client with connection pooling and retry logic
/// 
/// Provides a thread-safe, async Redis client with automatic connection
/// management and retry capabilities for resilient cache operations.
#[derive(Clone)]
pub struct RedisClient {
    /// Redis multiplexed connection for async operations
    connection: MultiplexedConnection,
    /// Configuration used to create this client
    config: CacheConfig,
    /// Maximum number of retry attempts for operations
    max_retries: u32,
    /// Base delay between retries (exponential backoff)
    retry_delay_ms: u64,
}

impl RedisClient {
    /// Create a new Redis client with connection pooling
    /// 
    /// # Arguments
    /// * `config` - Cache configuration settings
    /// 
    /// # Returns
    /// * `Result<Self, InfrastructureError>` - Redis client or error
    /// 
    /// # Example
    /// ```no_run
    /// use renov_infrastructure::config::CacheConfig;
    /// use renov_infrastructure::cache::redis_client::RedisClient;
    /// 
    /// async fn create_client() -> Result<RedisClient, Box<dyn std::error::Error>> {
    ///     let config = CacheConfig {
    ///         url: "redis://localhost:6379".to_string(),
    ///         pool_size: 10,
    ///         default_ttl: 3600,
    ///     };
    ///     let client = RedisClient::new(config).await?;
    ///     Ok(client)
    /// }
    /// ```
    pub async fn new(config: CacheConfig) -> Result<Self, InfrastructureError> {
        Self::new_with_retry_config(config, 3, 100).await
    }

    /// Create a new Redis client with custom retry configuration
    /// 
    /// # Arguments
    /// * `config` - Cache configuration settings
    /// * `max_retries` - Maximum number of retry attempts
    /// * `retry_delay_ms` - Base delay between retries in milliseconds
    /// 
    /// # Returns
    /// * `Result<Self, InfrastructureError>` - Redis client or error
    pub async fn new_with_retry_config(
        config: CacheConfig,
        max_retries: u32,
        retry_delay_ms: u64,
    ) -> Result<Self, InfrastructureError> {
        info!(
            "Creating Redis client with URL: {} and pool size: {}",
            mask_url(&config.url),
            config.pool_size
        );

        // Parse Redis URL and create client
        let client = Client::open(config.url.as_str()).map_err(|e| {
            error!("Failed to parse Redis URL: {}", e);
            InfrastructureError::Config(format!("Invalid Redis URL: {}", e))
        })?;

        // Create multiplexed connection with retry logic
        let connection = Self::create_connection_with_retry(
            client,
            max_retries,
            retry_delay_ms,
        ).await?;

        info!("Redis client created successfully");

        Ok(Self {
            connection,
            config,
            max_retries,
            retry_delay_ms,
        })
    }

    /// Create multiplexed connection with retry logic
    async fn create_connection_with_retry(
        client: Client,
        max_retries: u32,
        retry_delay_ms: u64,
    ) -> Result<MultiplexedConnection, InfrastructureError> {
        let mut attempts = 0;
        let mut delay = retry_delay_ms;

        loop {
            attempts += 1;
            debug!("Attempting to connect to Redis (attempt {})", attempts);

            match client.get_multiplexed_async_connection().await {
                Ok(connection) => {
                    info!("Successfully connected to Redis");
                    return Ok(connection);
                }
                Err(e) if attempts < max_retries => {
                    warn!(
                        "Failed to connect to Redis (attempt {}/{}): {}. Retrying in {}ms...",
                        attempts, max_retries, e, delay
                    );
                    sleep(Duration::from_millis(delay)).await;
                    // Exponential backoff with cap at 5 seconds
                    delay = (delay * 2).min(5000);
                }
                Err(e) => {
                    error!(
                        "Failed to connect to Redis after {} attempts: {}",
                        attempts, e
                    );
                    return Err(InfrastructureError::Cache(e));
                }
            }
        }
    }

    /// Set a value with expiration time
    /// 
    /// # Arguments
    /// * `key` - Cache key
    /// * `value` - Value to cache (must be serializable)
    /// * `expiry_seconds` - Time to live in seconds
    /// 
    /// # Returns
    /// * `Result<(), InfrastructureError>` - Success or error
    /// 
    /// # Example
    /// ```no_run
    /// use renov_infrastructure::cache::redis_client::RedisClient;
    /// 
    /// async fn cache_verification_code(client: &RedisClient) {
    ///     let phone = "1234567890";
    ///     let code = "123456";
    ///     let key = format!("verification:{}", phone);
    ///     
    ///     // Cache for 5 minutes (300 seconds)
    ///     client.set_with_expiry(&key, code, 300).await.unwrap();
    /// }
    /// ```
    pub async fn set_with_expiry(
        &self,
        key: &str,
        value: &str,
        expiry_seconds: u64,
    ) -> Result<(), InfrastructureError> {
        debug!("Setting key '{}' with expiry {}s", key, expiry_seconds);

        let result = self
            .execute_with_retry(|mut conn| {
                let key = key.to_string();
                let value = value.to_string();
                let expiry = expiry_seconds;
                
                Box::pin(async move {
                    conn.set_ex::<_, _, ()>(key, value, expiry).await
                })
            })
            .await;

        match result {
            Ok(_) => {
                debug!("Successfully set key '{}'", key);
                Ok(())
            }
            Err(e) => {
                error!("Failed to set key '{}': {}", key, e);
                Err(InfrastructureError::Cache(e))
            }
        }
    }

    /// Get a value from cache
    /// 
    /// # Arguments
    /// * `key` - Cache key
    /// 
    /// # Returns
    /// * `Result<Option<String>, InfrastructureError>` - Cached value or None if not found
    /// 
    /// # Example
    /// ```no_run
    /// use renov_infrastructure::cache::redis_client::RedisClient;
    /// 
    /// async fn get_verification_code(client: &RedisClient, phone: &str) {
    ///     let key = format!("verification:{}", phone);
    ///     
    ///     match client.get(&key).await {
    ///         Ok(Some(code)) => println!("Found code: {}", code),
    ///         Ok(None) => println!("Code not found or expired"),
    ///         Err(e) => println!("Error: {}", e),
    ///     }
    /// }
    /// ```
    pub async fn get(&self, key: &str) -> Result<Option<String>, InfrastructureError> {
        debug!("Getting key '{}'", key);

        let result = self
            .execute_with_retry(|mut conn| {
                let key = key.to_string();
                
                Box::pin(async move {
                    conn.get::<_, Option<String>>(key).await
                })
            })
            .await;

        match result {
            Ok(value) => {
                if value.is_some() {
                    debug!("Successfully retrieved key '{}'", key);
                } else {
                    debug!("Key '{}' not found", key);
                }
                Ok(value)
            }
            Err(e) => {
                error!("Failed to get key '{}': {}", key, e);
                Err(InfrastructureError::Cache(e))
            }
        }
    }

    /// Delete a key from cache
    /// 
    /// # Arguments
    /// * `key` - Cache key to delete
    /// 
    /// # Returns
    /// * `Result<bool, InfrastructureError>` - True if key was deleted, false if not found
    /// 
    /// # Example
    /// ```no_run
    /// use renov_infrastructure::cache::redis_client::RedisClient;
    /// 
    /// async fn invalidate_verification_code(client: &RedisClient, phone: &str) {
    ///     let key = format!("verification:{}", phone);
    ///     
    ///     match client.delete(&key).await {
    ///         Ok(true) => println!("Code deleted"),
    ///         Ok(false) => println!("Code was not found"),
    ///         Err(e) => println!("Error: {}", e),
    ///     }
    /// }
    /// ```
    pub async fn delete(&self, key: &str) -> Result<bool, InfrastructureError> {
        debug!("Deleting key '{}'", key);

        let result = self
            .execute_with_retry(|mut conn| {
                let key = key.to_string();
                
                Box::pin(async move {
                    conn.del::<_, u32>(key).await
                })
            })
            .await;

        match result {
            Ok(deleted_count) => {
                let deleted = deleted_count > 0;
                if deleted {
                    debug!("Successfully deleted key '{}'", key);
                } else {
                    debug!("Key '{}' was not found", key);
                }
                Ok(deleted)
            }
            Err(e) => {
                error!("Failed to delete key '{}': {}", key, e);
                Err(InfrastructureError::Cache(e))
            }
        }
    }

    /// Execute a Redis operation with automatic retry logic
    /// 
    /// This internal method provides retry capability for any Redis operation.
    /// It uses exponential backoff with the configured retry parameters.
    async fn execute_with_retry<F, T>(
        &self,
        operation: F,
    ) -> RedisResult<T>
    where
        F: Fn(MultiplexedConnection) -> std::pin::Pin<Box<dyn std::future::Future<Output = RedisResult<T>> + Send>>,
    {
        let mut attempts = 0;
        let mut delay = self.retry_delay_ms;

        loop {
            attempts += 1;
            let conn = self.connection.clone();

            match operation(conn).await {
                Ok(result) => return Ok(result),
                Err(e) if attempts < self.max_retries && is_retriable_error(&e) => {
                    warn!(
                        "Redis operation failed (attempt {}/{}): {}. Retrying in {}ms...",
                        attempts, self.max_retries, e, delay
                    );
                    sleep(Duration::from_millis(delay)).await;
                    // Exponential backoff with cap at 5 seconds
                    delay = (delay * 2).min(5000);
                }
                Err(e) => {
                    error!(
                        "Redis operation failed after {} attempts: {}",
                        attempts, e
                    );
                    return Err(e);
                }
            }
        }
    }

    /// Check if the Redis connection is healthy
    /// 
    /// Performs a PING command to verify connectivity.
    /// 
    /// # Returns
    /// * `Result<bool, InfrastructureError>` - True if healthy, error otherwise
    pub async fn health_check(&self) -> Result<bool, InfrastructureError> {
        debug!("Performing Redis health check");

        let result = self
            .execute_with_retry(|mut conn| {
                Box::pin(async move {
                    redis::cmd("PING").query_async::<_, String>(&mut conn).await
                })
            })
            .await;

        match result {
            Ok(response) if response == "PONG" => {
                debug!("Redis health check passed");
                Ok(true)
            }
            Ok(response) => {
                warn!("Redis health check returned unexpected response: {}", response);
                Ok(false)
            }
            Err(e) => {
                error!("Redis health check failed: {}", e);
                Err(InfrastructureError::Cache(e))
            }
        }
    }

    /// Increment a counter with optional expiry
    /// 
    /// Useful for rate limiting and counting operations.
    /// 
    /// # Arguments
    /// * `key` - Counter key
    /// * `expiry_seconds` - Optional expiry time in seconds
    /// 
    /// # Returns
    /// * `Result<i64, InfrastructureError>` - New counter value
    pub async fn increment(
        &self,
        key: &str,
        expiry_seconds: Option<u64>,
    ) -> Result<i64, InfrastructureError> {
        debug!("Incrementing counter '{}'", key);

        let result = self
            .execute_with_retry(|mut conn| {
                let key = key.to_string();
                let expiry = expiry_seconds;
                
                Box::pin(async move {
                    let count: i64 = conn.incr(&key, 1).await?;
                    
                    // Set expiry if this is the first increment
                    if count == 1 {
                        if let Some(ttl) = expiry {
                            conn.expire(&key, ttl as i64).await?;
                        }
                    }
                    
                    Ok(count)
                })
            })
            .await;

        match result {
            Ok(count) => {
                debug!("Counter '{}' incremented to {}", key, count);
                Ok(count)
            }
            Err(e) => {
                error!("Failed to increment counter '{}': {}", key, e);
                Err(InfrastructureError::Cache(e))
            }
        }
    }

    /// Check if a key exists in cache
    /// 
    /// # Arguments
    /// * `key` - Cache key
    /// 
    /// # Returns
    /// * `Result<bool, InfrastructureError>` - True if key exists
    pub async fn exists(&self, key: &str) -> Result<bool, InfrastructureError> {
        debug!("Checking if key '{}' exists", key);

        let result = self
            .execute_with_retry(|mut conn| {
                let key = key.to_string();
                
                Box::pin(async move {
                    conn.exists::<_, bool>(key).await
                })
            })
            .await;

        match result {
            Ok(exists) => {
                debug!("Key '{}' exists: {}", key, exists);
                Ok(exists)
            }
            Err(e) => {
                error!("Failed to check key '{}' existence: {}", key, e);
                Err(InfrastructureError::Cache(e))
            }
        }
    }

    /// Get time-to-live for a key
    /// 
    /// # Arguments
    /// * `key` - Cache key
    /// 
    /// # Returns
    /// * `Result<Option<i64>, InfrastructureError>` - TTL in seconds, None if key doesn't exist or has no expiry
    pub async fn ttl(&self, key: &str) -> Result<Option<i64>, InfrastructureError> {
        debug!("Getting TTL for key '{}'", key);

        let result = self
            .execute_with_retry(|mut conn| {
                let key = key.to_string();
                
                Box::pin(async move {
                    conn.ttl::<_, i64>(key).await
                })
            })
            .await;

        match result {
            Ok(ttl) if ttl >= 0 => {
                debug!("Key '{}' has TTL: {}s", key, ttl);
                Ok(Some(ttl))
            }
            Ok(ttl) if ttl == -1 => {
                debug!("Key '{}' exists but has no expiry", key);
                Ok(None)
            }
            Ok(_) => {
                debug!("Key '{}' does not exist", key);
                Ok(None)
            }
            Err(e) => {
                error!("Failed to get TTL for key '{}': {}", key, e);
                Err(InfrastructureError::Cache(e))
            }
        }
    }
}

/// Check if a Redis error is retriable
/// 
/// Determines if an error is transient and the operation should be retried.
#[cfg(test)]
pub(crate) fn is_retriable_error(error: &RedisError) -> bool {
    matches!(
        error.kind(),
        redis::ErrorKind::IoError
            | redis::ErrorKind::ClientError
            | redis::ErrorKind::BusyLoadingError
            | redis::ErrorKind::TryAgain
    )
}

#[cfg(not(test))]
fn is_retriable_error(error: &RedisError) -> bool {
    matches!(
        error.kind(),
        redis::ErrorKind::IoError
            | redis::ErrorKind::ClientError
            | redis::ErrorKind::BusyLoadingError
            | redis::ErrorKind::TryAgain
    )
}

/// Mask sensitive parts of Redis URL for logging
#[cfg(test)]
pub(crate) fn mask_url(url: &str) -> String {
    if let Some(at_pos) = url.find('@') {
        if let Some(proto_end) = url.find("://") {
            let proto = &url[..proto_end + 3];
            let host_part = &url[at_pos..];
            return format!("{}****{}", proto, host_part);
        }
    }
    url.to_string()
}

#[cfg(not(test))]
fn mask_url(url: &str) -> String {
    if let Some(at_pos) = url.find('@') {
        if let Some(proto_end) = url.find("://") {
            let proto = &url[..proto_end + 3];
            let host_part = &url[at_pos..];
            return format!("{}****{}", proto, host_part);
        }
    }
    url.to_string()
}