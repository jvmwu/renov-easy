//! Repository trait definitions for data access abstraction.

pub mod user_repository;
pub mod token_repository;

// Re-export commonly used types
pub use user_repository::UserRepository;
pub use token_repository::TokenRepository;

// Placeholder for future repository trait modules
// pub mod order_repository;