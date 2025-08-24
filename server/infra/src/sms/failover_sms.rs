//! Failover SMS Service Implementation
//!
//! This module provides an SMS service that automatically fails over from a primary
//! provider to a backup provider when the primary service is unavailable.
//!
//! ## Features
//!
//! - Automatic failover from primary to backup SMS provider
//! - Configurable failover timeout (default: 30 seconds)
//! - Health check monitoring for automatic recovery
//! - Seamless switching between providers
//! - Comprehensive logging of failover events

use async_trait::async_trait;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::{
    sms::sms_service::SmsService,
    InfrastructureError,
};
use re_core::services::verification::SmsServiceTrait;

/// State tracking for failover service
#[derive(Debug, Clone)]
struct FailoverState {
    /// Whether we're currently using the backup service
    using_backup: bool,
    /// When the primary service last failed
    last_primary_failure: Option<Instant>,
    /// Number of consecutive failures on primary
    primary_failure_count: u32,
}

impl Default for FailoverState {
    fn default() -> Self {
        Self {
            using_backup: false,
            last_primary_failure: None,
            primary_failure_count: 0,
        }
    }
}

/// SMS service with automatic failover capability
pub struct FailoverSmsService {
    /// Primary SMS service (e.g., Twilio)
    primary: Box<dyn SmsService>,
    /// Backup SMS service (e.g., AWS SNS)
    backup: Box<dyn SmsService>,
    /// Failover state
    state: Arc<RwLock<FailoverState>>,
    /// How long to wait before retrying primary after failure
    failover_timeout: Duration,
}

impl FailoverSmsService {
    /// Create a new failover SMS service
    ///
    /// # Arguments
    ///
    /// * `primary` - The primary SMS service to use
    /// * `backup` - The backup SMS service to fail over to
    /// * `failover_timeout` - How long to wait before retrying the primary service
    pub fn new(
        primary: Box<dyn SmsService>,
        backup: Box<dyn SmsService>,
        failover_timeout: Duration,
    ) -> Self {
        info!(
            "Initializing failover SMS service with {} (primary) and {} (backup)",
            primary.provider_name(),
            backup.provider_name()
        );
        
        Self {
            primary,
            backup,
            state: Arc::new(RwLock::new(FailoverState::default())),
            failover_timeout,
        }
    }
    
    /// Check if we should try the primary service again
    async fn should_retry_primary(&self) -> bool {
        let state = self.state.read().await;
        
        if !state.using_backup {
            return true; // Already using primary
        }
        
        if let Some(last_failure) = state.last_primary_failure {
            // Check if enough time has passed since last failure
            last_failure.elapsed() > self.failover_timeout
        } else {
            true // No recorded failure, try primary
        }
    }
    
    /// Record a primary service failure and switch to backup
    async fn record_primary_failure(&self) {
        let mut state = self.state.write().await;
        
        state.primary_failure_count += 1;
        state.last_primary_failure = Some(Instant::now());
        
        if !state.using_backup {
            warn!(
                "Primary SMS service ({}) failed, switching to backup ({})",
                self.primary.provider_name(),
                self.backup.provider_name()
            );
            state.using_backup = true;
        }
    }
    
    /// Record a successful primary service operation
    async fn record_primary_success(&self) {
        let mut state = self.state.write().await;
        
        if state.using_backup {
            info!(
                "Primary SMS service ({}) recovered, switching back from backup",
                self.primary.provider_name()
            );
        }
        
        state.using_backup = false;
        state.primary_failure_count = 0;
        state.last_primary_failure = None;
    }
}

#[async_trait]
impl SmsService for FailoverSmsService {
    async fn send_sms(&self, phone_number: &str, message: &str) -> Result<String, InfrastructureError> {
        // Check if we should try the primary service
        if self.should_retry_primary().await {
            // Try primary service first
            match self.primary.send_sms(phone_number, message).await {
                Ok(result) => {
                    self.record_primary_success().await;
                    return Ok(result);
                }
                Err(e) => {
                    error!(
                        "Primary SMS service ({}) failed: {}",
                        self.primary.provider_name(),
                        e
                    );
                    self.record_primary_failure().await;
                }
            }
        }
        
        // Use backup service
        info!(
            "Using backup SMS service ({}) to send message",
            self.backup.provider_name()
        );
        
        match self.backup.send_sms(phone_number, message).await {
            Ok(result) => Ok(result),
            Err(e) => {
                error!(
                    "Backup SMS service ({}) also failed: {}",
                    self.backup.provider_name(),
                    e
                );
                Err(InfrastructureError::Sms(format!(
                    "Both primary and backup SMS services failed. Primary: {}, Backup: {}",
                    self.primary.provider_name(),
                    self.backup.provider_name()
                )))
            }
        }
    }
    
    fn provider_name(&self) -> &str {
        "Failover"
    }
    
    async fn is_available(&self) -> bool {
        // Check if either service is available
        let primary_available = self.primary.is_available().await;
        let backup_available = self.backup.is_available().await;
        
        if !primary_available && backup_available {
            // Record that primary is down
            self.record_primary_failure().await;
        } else if primary_available {
            // Record that primary is up
            self.record_primary_success().await;
        }
        
        primary_available || backup_available
    }
}

/// Adapter that implements the core SmsServiceTrait for the failover service
pub struct FailoverSmsServiceAdapter {
    inner: Arc<FailoverSmsService>,
}

impl FailoverSmsServiceAdapter {
    /// Create a new failover SMS service adapter
    pub fn new(
        primary: Box<dyn SmsService>,
        backup: Box<dyn SmsService>,
        failover_timeout: Duration,
    ) -> Self {
        Self {
            inner: Arc::new(FailoverSmsService::new(primary, backup, failover_timeout)),
        }
    }
}

#[async_trait]
impl SmsServiceTrait for FailoverSmsServiceAdapter {
    async fn send_verification_code(&self, phone: &str, code: &str) -> Result<String, String> {
        match self.inner.send_verification_code(phone, code).await {
            Ok(message_id) => Ok(message_id),
            Err(e) => Err(e.to_string()),
        }
    }
    
    fn is_valid_phone_number(&self, phone: &str) -> bool {
        crate::sms::sms_service::is_valid_phone_number(phone)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sms::mock_sms::MockSmsService;
    
    #[tokio::test]
    async fn test_failover_to_backup() {
        // Create a mock service that always fails as primary
        struct FailingService;
        
        #[async_trait]
        impl SmsService for FailingService {
            async fn send_sms(&self, _: &str, _: &str) -> Result<String, InfrastructureError> {
                Err(InfrastructureError::Sms("Primary service failed".to_string()))
            }
            
            fn provider_name(&self) -> &str {
                "FailingPrimary"
            }
            
            async fn is_available(&self) -> bool {
                false
            }
        }
        
        let primary = Box::new(FailingService);
        let backup = Box::new(MockSmsService::new());
        
        let service = FailoverSmsService::new(
            primary,
            backup,
            Duration::from_secs(30),
        );
        
        // Should fail over to backup
        let result = service.send_sms("+1234567890", "Test message").await;
        assert!(result.is_ok());
        
        // Check that we're using backup
        let state = service.state.read().await;
        assert!(state.using_backup);
    }
    
    #[tokio::test]
    async fn test_primary_recovery() {
        // Create services
        let primary = Box::new(MockSmsService::new());
        let backup = Box::new(MockSmsService::new());
        
        let service = FailoverSmsService::new(
            primary,
            backup,
            Duration::from_millis(100), // Short timeout for testing
        );
        
        // Force a failure state
        {
            let mut state = service.state.write().await;
            state.using_backup = true;
            state.last_primary_failure = Some(Instant::now() - Duration::from_secs(1));
        }
        
        // Should recover to primary after timeout
        let result = service.send_sms("+1234567890", "Test message").await;
        assert!(result.is_ok());
        
        // Check that we're back to primary
        let state = service.state.read().await;
        assert!(!state.using_backup);
    }
    
    #[tokio::test]
    async fn test_both_services_fail() {
        struct FailingService;
        
        #[async_trait]
        impl SmsService for FailingService {
            async fn send_sms(&self, _: &str, _: &str) -> Result<String, InfrastructureError> {
                Err(InfrastructureError::Sms("Service failed".to_string()))
            }
            
            fn provider_name(&self) -> &str {
                "Failing"
            }
        }
        
        let primary = Box::new(FailingService);
        let backup = Box::new(FailingService);
        
        let service = FailoverSmsService::new(
            primary,
            backup,
            Duration::from_secs(30),
        );
        
        // Should fail when both services fail
        let result = service.send_sms("+1234567890", "Test message").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Both primary and backup"));
    }
}