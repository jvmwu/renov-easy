pub mod r#trait {
    pub use super::trait_::*;
}
#[path = "trait.rs"]
mod trait_;
pub mod repository;

pub use r#trait::TokenRepository;
pub use repository::MySqlTokenRepository;

#[cfg(test)]
pub mod mock;
#[cfg(test)]
pub use mock::MockTokenRepository;

#[cfg(test)]
mod tests;