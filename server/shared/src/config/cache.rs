//! Cache configuration module

use serde::{Deserialize, Serialize};

/// Redis cache configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CacheConfig {
    /// Redis connection URL
    pub url: String,
    
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    
    /// Response timeout in seconds
    pub response_timeout: u64,
    
    /// Default TTL for cache entries in seconds
    #[serde(default = "default_ttl")]
    pub default_ttl: u64,
    
    /// Enable cache key prefix
    #[serde(default)]
    pub key_prefix: Option<String>,
    
    /// Redis database number (0-15)
    #[serde(default)]
    pub database: u8,
    
    /// Enable cache statistics
    #[serde(default)]
    pub enable_stats: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            url: String::from("redis://localhost:6379"),
            max_connections: 10,
            connection_timeout: 5,
            response_timeout: 5,
            default_ttl: default_ttl(),
            key_prefix: None,
            database: 0,
            enable_stats: false,
        }
    }
}

impl CacheConfig {
    /// Create from environment variables
    pub fn from_env() -> Self {
        let url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());
        let max_connections = std::env::var("REDIS_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10);
            
        Self {
            url,
            max_connections,
            ..Default::default()
        }
    }
    
    /// Create a new cache configuration with URL
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }
    
    /// Get Redis URL (backward compatibility)
    pub fn redis_url(&self) -> &str {
        &self.url
    }
    
    /// Get pool size (backward compatibility)
    pub fn pool_size(&self) -> u32 {
        self.max_connections
    }
    
    /// Get default TTL in seconds (backward compatibility)
    pub fn default_ttl_seconds(&self) -> u64 {
        self.default_ttl
    }
    
    /// Set the key prefix for all cache keys
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.key_prefix = Some(prefix.into());
        self
    }
    
    /// Set the database number
    pub fn with_database(mut self, db: u8) -> Self {
        self.database = db.min(15);
        self
    }
    
    /// Generate a cache key with prefix
    pub fn make_key(&self, key: &str) -> String {
        match &self.key_prefix {
            Some(prefix) => format!("{}:{}", prefix, key),
            None => key.to_string(),
        }
    }
}

/// In-memory cache configuration (for development/testing)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MemoryCacheConfig {
    /// Maximum number of entries
    pub max_entries: usize,
    
    /// Default TTL for entries in seconds
    pub default_ttl: u64,
    
    /// Enable LRU eviction
    #[serde(default = "default_lru")]
    pub enable_lru: bool,
    
    /// Cleanup interval in seconds
    #[serde(default = "default_cleanup_interval")]
    pub cleanup_interval: u64,
}

impl Default for MemoryCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            default_ttl: 300,  // 5 minutes
            enable_lru: default_lru(),
            cleanup_interval: default_cleanup_interval(),
        }
    }
}

/// Cache strategy configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CacheStrategyConfig {
    /// Enable caching
    pub enabled: bool,
    
    /// Cache type (redis, memory, hybrid)
    #[serde(default = "default_cache_type")]
    pub cache_type: CacheType,
    
    /// Redis configuration
    #[serde(default)]
    pub redis: Option<CacheConfig>,
    
    /// Memory cache configuration
    #[serde(default)]
    pub memory: Option<MemoryCacheConfig>,
}

/// Cache type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CacheType {
    Redis,
    Memory,
    Hybrid,  // Use memory as L1 cache, Redis as L2
}

impl Default for CacheStrategyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cache_type: default_cache_type(),
            redis: Some(CacheConfig::default()),
            memory: Some(MemoryCacheConfig::default()),
        }
    }
}

fn default_ttl() -> u64 {
    3600  // 1 hour
}

fn default_lru() -> bool {
    true
}

fn default_cleanup_interval() -> u64 {
    60  // 1 minute
}

fn default_cache_type() -> CacheType {
    CacheType::Redis
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.url, "redis://localhost:6379");
        assert_eq!(config.database, 0);
        assert_eq!(config.default_ttl, 3600);
    }
    
    #[test]
    fn test_cache_config_with_prefix() {
        let config = CacheConfig::new("redis://cache:6379")
            .with_prefix("renoveasy")
            .with_database(2);
        
        assert_eq!(config.make_key("user:123"), "renoveasy:user:123");
        assert_eq!(config.database, 2);
    }
    
    #[test]
    fn test_cache_key_without_prefix() {
        let config = CacheConfig::default();
        assert_eq!(config.make_key("user:123"), "user:123");
    }
    
    #[test]
    fn test_memory_cache_config() {
        let config = MemoryCacheConfig::default();
        assert_eq!(config.max_entries, 1000);
        assert!(config.enable_lru);
        assert_eq!(config.cleanup_interval, 60);
    }
    
    #[test]
    fn test_cache_strategy() {
        let config = CacheStrategyConfig::default();
        assert!(config.enabled);
        assert_eq!(config.cache_type, CacheType::Redis);
        assert!(config.redis.is_some());
        assert!(config.memory.is_some());
    }
}