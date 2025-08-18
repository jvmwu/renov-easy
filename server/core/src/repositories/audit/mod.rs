//! Audit log repository module.

mod r#trait;
pub use r#trait::AuditLogRepository;

mod repository;
pub use repository::MySqlAuditLogRepository;

#[cfg(test)]
mod mock;
#[cfg(test)]
pub use mock::MockAuditLogRepository;

#[cfg(test)]
mod tests;