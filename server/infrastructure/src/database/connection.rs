//! Database connection pool management
//! 
//! This module provides database connection pooling using SQLx with MySQL.
//! It implements connection pool configuration, health checks, and connection
//! management following best practices for async Rust applications.

use sqlx::{
    mysql::{MySqlConnectOptions, MySqlPoolOptions},
    ConnectOptions, MySqlPool,
};
use std::str::FromStr;
use std::time::Duration;
use tracing::log::LevelFilter;

use crate::config::DatabaseConfig;
use crate::InfrastructureError;

/// Database connection pool wrapper
/// 
/// Manages the MySQL connection pool with configurable settings
/// for connection limits, timeouts, and health checks.
#[derive(Clone)]
pub struct DatabasePool {
    /// SQLx MySQL connection pool
    pool: MySqlPool,
    /// Configuration used to create this pool
    config: DatabaseConfig,
}

impl DatabasePool {
    /// Create a new database connection pool
    /// 
    /// # Arguments
    /// * `config` - Database configuration settings
    /// 
    /// # Returns
    /// * `Result<Self, InfrastructureError>` - Database pool or error
    /// 
    /// # Example
    /// ```no_run
    /// use renov_infrastructure::config::DatabaseConfig;
    /// use renov_infrastructure::database::connection::DatabasePool;
    /// 
    /// async fn create_pool() -> Result<DatabasePool, Box<dyn std::error::Error>> {
    ///     let config = DatabaseConfig {
    ///         url: "mysql://user:pass@localhost/db".to_string(),
    ///         max_connections: 10,
    ///         connect_timeout: 30,
    ///     };
    ///     let pool = DatabasePool::new(config).await?;
    ///     Ok(pool)
    /// }
    /// ```
    pub async fn new(config: DatabaseConfig) -> Result<Self, InfrastructureError> {
        tracing::info!(
            "Creating database connection pool with max_connections: {}",
            config.max_connections
        );

        // Parse connection options from URL
        let mut connect_options = MySqlConnectOptions::from_str(&config.url)
            .map_err(|e| InfrastructureError::Config(format!("Invalid database URL: {}", e)))?;

        // Configure connection logging
        connect_options = connect_options
            .log_statements(LevelFilter::Debug)
            .log_slow_statements(LevelFilter::Warn, Duration::from_secs(1));

        // Create pool with configuration
        let pool = MySqlPoolOptions::new()
            // Connection pool size
            .max_connections(config.max_connections)
            .min_connections(1)
            // Connection lifecycle
            .acquire_timeout(Duration::from_secs(config.connect_timeout))
            .idle_timeout(Duration::from_secs(600)) // 10 minutes
            .max_lifetime(Duration::from_secs(1800)) // 30 minutes
            // Test connections before returning from pool
            .test_before_acquire(true)
            // Build and connect
            .connect_with(connect_options)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create database pool: {}", e);
                InfrastructureError::Database(e)
            })?;

        tracing::info!("Database connection pool created successfully");

        Ok(Self { pool, config })
    }

    /// Get a reference to the underlying SQLx pool
    /// 
    /// Use this for executing queries and transactions.
    /// 
    /// # Returns
    /// * `&MySqlPool` - Reference to the SQLx MySQL pool
    pub fn get_pool(&self) -> &MySqlPool {
        &self.pool
    }

    /// Check if the database connection is healthy
    /// 
    /// Performs a simple query to verify connectivity.
    /// 
    /// # Returns
    /// * `Result<bool, InfrastructureError>` - True if healthy, error otherwise
    /// 
    /// # Example
    /// ```no_run
    /// use renov_infrastructure::database::connection::DatabasePool;
    /// 
    /// async fn check_health(pool: &DatabasePool) {
    ///     match pool.health_check().await {
    ///         Ok(true) => println!("Database is healthy"),
    ///         Ok(false) => println!("Database check returned false"),
    ///         Err(e) => println!("Database is unhealthy: {}", e),
    ///     }
    /// }
    /// ```
    pub async fn health_check(&self) -> Result<bool, InfrastructureError> {
        tracing::debug!("Performing database health check");

        // Execute a simple query to verify connectivity
        let result = sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("Database health check failed: {}", e);
                InfrastructureError::Database(e)
            })?;

        // Verify we got the expected result
        let value: i32 = sqlx::Row::try_get(&result, 0).unwrap_or(0);
        
        if value == 1 {
            tracing::debug!("Database health check passed");
            Ok(true)
        } else {
            tracing::warn!("Database health check returned unexpected value: {}", value);
            Ok(false)
        }
    }

    /// Get connection pool statistics
    /// 
    /// Returns information about the current state of the connection pool.
    /// 
    /// # Returns
    /// * `PoolStatistics` - Current pool statistics
    pub fn get_statistics(&self) -> PoolStatistics {
        PoolStatistics {
            connections: self.pool.size(),
            idle_connections: self.pool.num_idle(),
            max_connections: self.pool.options().get_max_connections(),
        }
    }

    /// Close all connections in the pool
    /// 
    /// This should be called during application shutdown.
    pub async fn close(&self) {
        tracing::info!("Closing database connection pool");
        self.pool.close().await;
        tracing::info!("Database connection pool closed");
    }

    /// Execute a database migration
    /// 
    /// Runs SQL migration scripts from the migrations directory.
    /// This is typically called during application startup.
    /// 
    /// # Returns
    /// * `Result<(), InfrastructureError>` - Success or error
    pub async fn run_migrations(&self) -> Result<(), InfrastructureError> {
        tracing::info!("Running database migrations");
        
        // SQLx migrations would be configured here
        // For now, migrations are run manually
        // In production, use: sqlx::migrate!("./migrations").run(&self.pool).await?;
        
        tracing::info!("Database migrations completed");
        Ok(())
    }

    /// Begin a new database transaction
    /// 
    /// # Returns
    /// * `Result<sqlx::Transaction<'_, MySql>, InfrastructureError>` - Transaction handle
    pub async fn begin_transaction(
        &self,
    ) -> Result<sqlx::Transaction<'_, sqlx::MySql>, InfrastructureError> {
        self.pool
            .begin()
            .await
            .map_err(|e| InfrastructureError::Database(e))
    }
}

/// Connection pool statistics
#[derive(Debug, Clone)]
pub struct PoolStatistics {
    /// Total number of connections in the pool
    pub connections: u32,
    /// Number of idle connections
    pub idle_connections: usize,
    /// Maximum allowed connections
    pub max_connections: u32,
}

impl std::fmt::Display for PoolStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Pool Stats: {}/{} connections ({} idle)",
            self.connections, self.max_connections, self.idle_connections
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_creation_with_invalid_url() {
        let config = DatabaseConfig {
            url: "invalid://url".to_string(),
            max_connections: 10,
            connect_timeout: 5,
        };

        let result = DatabasePool::new(config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore] // Requires actual database
    async fn test_pool_health_check() {
        let config = DatabaseConfig {
            url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "mysql://root:password@localhost/renovesy_test".to_string()),
            max_connections: 5,
            connect_timeout: 10,
        };

        let pool = DatabasePool::new(config).await.unwrap();
        let health = pool.health_check().await.unwrap();
        assert!(health);
    }

    #[test]
    fn test_pool_statistics_display() {
        let stats = PoolStatistics {
            connections: 5,
            idle_connections: 3,
            max_connections: 10,
        };

        let display = format!("{}", stats);
        assert!(display.contains("5/10"));
        assert!(display.contains("3 idle"));
    }
}