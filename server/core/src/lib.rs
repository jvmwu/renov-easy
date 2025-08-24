//! # RenovEasy Core
//!
//! Core business logic and domain layer for the RenovEasy backend.
//! This crate contains domain entities, business services, repository interfaces,
//! and error types that form the foundation of the application architecture.

pub mod domain;
pub mod services;
pub mod repositories;
pub mod errors;

// Re-export specific types to avoid naming conflicts
// Domain exports
pub use domain::entities;
pub use domain::value_objects;

// Service exports
pub use services::auth::{AuthService, AuthServiceConfig, RateLimiterTrait};

// Repository exports
pub use repositories::user::UserRepository;
pub use repositories::token::TokenRepository;

// Error exports
pub use errors::{DomainError, DomainResult};
pub use errors::{AuthError, TokenError, ValidationError, ErrorResponse};
