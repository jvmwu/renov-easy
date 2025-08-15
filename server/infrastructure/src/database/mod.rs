//! Database module - MySQL implementations using SQLx
//! 
//! This module provides database access layer implementations including:
//! - Connection pool management
//! - Repository pattern implementations
//! - Transaction support
//! - Database migrations

pub mod connection;

// Re-export commonly used types
pub use connection::{DatabasePool, PoolStatistics};

// Future modules will be added here:
// pub mod mysql;
// pub mod repositories;