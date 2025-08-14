//! Domain-specific error types and error handling.

use thiserror::Error;

/// Core domain errors
#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Validation error: {message}")]
    Validation { message: String },
    
    #[error("Business rule violation: {message}")]
    BusinessRule { message: String },
    
    #[error("Resource not found: {resource}")]
    NotFound { resource: String },
    
    #[error("Unauthorized access")]
    Unauthorized,
    
    #[error("Internal error: {message}")]
    Internal { message: String },
}

pub type DomainResult<T> = Result<T, DomainError>;