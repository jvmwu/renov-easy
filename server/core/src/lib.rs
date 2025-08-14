//! # RenovEasy Core
//! 
//! Core business logic and domain layer for the RenovEasy backend.
//! This crate contains domain entities, business services, repository interfaces,
//! and error types that form the foundation of the application architecture.

pub mod domain;
pub mod services;
pub mod repositories;
pub mod errors;

// Re-export commonly used types for convenience
pub use domain::*;
pub use services::*;
pub use repositories::*;
pub use errors::*;