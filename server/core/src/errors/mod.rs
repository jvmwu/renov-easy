//! Domain-specific error types and error handling.

mod types;

// Re-export all error types and utilities
pub use types::{
    AuthError, DomainErrorResponse as ErrorResponse, TokenError, ValidationError,
    extract_chinese_message, extract_english_message,
};

use thiserror::Error;

/// Core domain errors (general purpose)
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
    
    // Bridge to specific error types
    #[error(transparent)]
    Auth(#[from] AuthError),
    
    #[error(transparent)]
    Token(#[from] TokenError),
    
    #[error(transparent)]
    ValidationErr(#[from] ValidationError),
}

pub type DomainResult<T> = Result<T, DomainError>;