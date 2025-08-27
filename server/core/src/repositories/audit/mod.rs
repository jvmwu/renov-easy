//! Audit log repository module.

mod r#trait;
pub use r#trait::AuditLogRepository;

mod repository;
pub use repository::MySqlAuditLogRepository;

mod noop;
pub use noop::NoOpAuditLogRepository;

mod mock;
pub use mock::MockAuditLogRepository;