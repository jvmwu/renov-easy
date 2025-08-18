//! Audit service module for recording authentication attempts and security events.

mod service;

pub use service::{AuditService, AuditServiceConfig};

#[cfg(test)]
mod tests;