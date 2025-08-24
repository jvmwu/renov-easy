pub mod r#trait {
    pub use super::trait_::*;
}
#[path = "trait.rs"]
mod trait_;
pub mod repository;

pub use r#trait::UserRepository;
pub use repository::MySqlUserRepository;