//! Database module - MySQL implementations using SQLx
//! 
//! This module provides database access layer implementations including:
//! - Connection pool management
//! - Repository pattern implementations
//! - Transaction support
//! - Database migrations

pub mod connection;
pub mod mysql;
pub mod repositories;

#[cfg(test)]
mod tests;

// Re-export commonly used types
pub use connection::{DatabasePool, PoolStatistics};
pub use mysql::{MySqlUserRepository, MySqlTokenRepository};
pub use repositories::OtpRepository;