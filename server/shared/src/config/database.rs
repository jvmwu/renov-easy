//! Database configuration module

use serde::{Deserialize, Serialize};

/// Database configuration for MySQL/PostgreSQL connections
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    /// Database connection URL
    pub url: String,

    /// Maximum number of connections in the pool
    pub max_connections: u32,

    /// Connection timeout in seconds
    pub connect_timeout: u64,

    /// Idle connection timeout in seconds
    pub idle_timeout: u64,

    /// Maximum lifetime of a connection in seconds
    pub max_lifetime: u64,

    /// Enable SQL query logging
    #[serde(default)]
    pub enable_logging: bool,

    /// Slow query threshold in milliseconds
    #[serde(default = "default_slow_query_threshold")]
    pub slow_query_threshold: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: String::from("mysql://localhost:3306/renoveasy"),
            max_connections: 10,
            connect_timeout: 30,
            idle_timeout: 600,
            max_lifetime: 1800,
            enable_logging: false,
            slow_query_threshold: default_slow_query_threshold(),
        }
    }
}

impl DatabaseConfig {
    /// Create from environment variables
    pub fn from_env() -> Self {
        let url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "mysql://root:password@localhost:3306/renoveasy".to_string());
        let max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10);
        let connect_timeout = std::env::var("DATABASE_CONNECT_TIMEOUT")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30);

        Self {
            url,
            max_connections,
            connect_timeout,
            ..Default::default()
        }
    }

    /// Create a new database configuration with URL
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }

    /// Set the maximum number of connections
    pub fn with_max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    /// Enable SQL query logging
    pub fn with_logging(mut self, enable: bool) -> Self {
        self.enable_logging = enable;
        self
    }

    /// Check if this is a production database
    pub fn is_production(&self) -> bool {
        !self.url.contains("localhost") && !self.url.contains("127.0.0.1")
    }
}

fn default_slow_query_threshold() -> u64 {
    1000 // 1 second
}
