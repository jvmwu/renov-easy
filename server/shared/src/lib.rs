//! Shared utilities and common types for RenovEasy server
//!
//! This crate provides common functionality used across all server modules:
//! - Configuration types
//! - Error types and response structures
//! - Utility functions (phone validation, etc.)
//! - Common type definitions

pub mod config;
pub mod errors;
pub mod types;
pub mod utils;

// Re-export commonly used items at crate root
pub use config::{
    AppConfig, Environment,
    DatabaseConfig, JwtConfig, CacheConfig, RateLimitConfig,
    ServerConfig, CorsConfig, AuthConfig, LoggingConfig
};
pub use errors::{ErrorResponse, IntoErrorResponse, ApiResult, error_codes};
pub use types::{
    Language, Pagination, PaginatedResponse, ApiResponse,
    Id, Status, Priority, Coordinate, DateRange
};
pub use utils::{phone, validation};
