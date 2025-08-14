//! Domain layer containing business entities, value objects, and domain events.

pub mod entities;
pub mod value_objects;
pub mod events;

// Re-export commonly used domain types
pub use entities::*;
pub use value_objects::*;
pub use events::*;