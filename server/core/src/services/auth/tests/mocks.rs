//! Mock implementations for testing authentication service

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::domain::entities::user::{User, UserType};
use crate::errors::{AuthError, DomainError};
use crate::repositories::UserRepository;
use crate::services::verification::{CacheServiceTrait, SmsServiceTrait};
use crate::services::auth::rate_limiter::RateLimiterTrait;

pub struct MockUserRepository {
    pub users: Arc<Mutex<Vec<User>>>,
}

impl MockUserRepository {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    pub fn with_existing_user(user: User) -> Self {
        let repo = Self::new();
        repo.users.lock().unwrap().push(user);
        repo
    }
}

#[async_trait]
impl UserRepository for MockUserRepository {
    async fn find_by_phone(
        &self,
        phone_hash: &str,
        country_code: &str,
    ) -> Result<Option<User>, DomainError> {
        let users = self.users.lock().unwrap();
        Ok(users.iter()
            .find(|u| u.phone_hash == phone_hash && u.country_code == country_code)
            .cloned())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError> {
        let users = self.users.lock().unwrap();
        Ok(users.iter().find(|u| u.id == id).cloned())
    }

    async fn create(&self, user: User) -> Result<User, DomainError> {
        let mut users = self.users.lock().unwrap();
        // Check for duplicate
        if users.iter().any(|u| u.phone_hash == user.phone_hash && u.country_code == user.country_code) {
            return Err(DomainError::Auth(AuthError::UserAlreadyExists));
        }
        users.push(user.clone());
        Ok(user)
    }

    async fn update(&self, user: User) -> Result<User, DomainError> {
        let mut users = self.users.lock().unwrap();
        if let Some(existing) = users.iter_mut().find(|u| u.id == user.id) {
            *existing = user.clone();
            Ok(user)
        } else {
            Err(DomainError::Auth(AuthError::UserNotFound))
        }
    }

    async fn exists_by_phone(
        &self,
        phone_hash: &str,
        country_code: &str,
    ) -> Result<bool, DomainError> {
        let users = self.users.lock().unwrap();
        Ok(users.iter().any(|u| u.phone_hash == phone_hash && u.country_code == country_code))
    }

    async fn count_by_type(&self, user_type: Option<UserType>) -> Result<u64, DomainError> {
        let users = self.users.lock().unwrap();
        let count = match user_type {
            Some(ut) => users.iter().filter(|u| u.user_type == Some(ut)).count(),
            None => users.len(),
        };
        Ok(count as u64)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, DomainError> {
        let mut users = self.users.lock().unwrap();
        if let Some(index) = users.iter().position(|u| u.id == id) {
            users.remove(index);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

pub struct MockSmsService;

#[async_trait]
impl SmsServiceTrait for MockSmsService {
    async fn send_verification_code(&self, _phone: &str, _code: &str) -> Result<String, String> {
        Ok("mock-message-id".to_string())
    }

    fn is_valid_phone_number(&self, phone: &str) -> bool {
        phone.starts_with('+') && phone.len() >= 10
    }
}

pub struct MockCacheService {
    pub verify_success: bool,
    pub remaining_attempts: i64,
}

impl MockCacheService {
    pub fn new_success() -> Self {
        Self {
            verify_success: true,
            remaining_attempts: 3,
        }
    }

    pub fn new_failure(remaining_attempts: i64) -> Self {
        Self {
            verify_success: false,
            remaining_attempts,
        }
    }
}

#[async_trait]
impl CacheServiceTrait for MockCacheService {
    async fn store_code(&self, _phone: &str, _code: &str) -> Result<(), String> {
        Ok(())
    }

    async fn verify_code(&self, _phone: &str, _code: &str) -> Result<bool, String> {
        Ok(self.verify_success)
    }

    async fn get_remaining_attempts(&self, _phone: &str) -> Result<i64, String> {
        Ok(self.remaining_attempts)
    }

    async fn code_exists(&self, _phone: &str) -> Result<bool, String> {
        Ok(false)
    }

    async fn get_code_ttl(&self, _phone: &str) -> Result<Option<i64>, String> {
        Ok(None)
    }

    async fn clear_verification(&self, _phone: &str) -> Result<(), String> {
        Ok(())
    }
}

pub struct MockRateLimiter {
    pub phone_counters: Arc<Mutex<HashMap<String, i64>>>,
    pub ip_counters: Arc<Mutex<HashMap<String, i64>>>,
    pub max_requests: i64,
    pub max_ip_attempts: i64,
    pub rate_limit_logs: Arc<Mutex<Vec<(String, String, String)>>>,
}

impl MockRateLimiter {
    pub fn new(max_requests: i64) -> Self {
        Self {
            phone_counters: Arc::new(Mutex::new(HashMap::new())),
            ip_counters: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            max_ip_attempts: 10, // Default max IP attempts
            rate_limit_logs: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl RateLimiterTrait for MockRateLimiter {
    async fn check_sms_rate_limit(&self, phone: &str) -> Result<bool, String> {
        let counters = self.phone_counters.lock().unwrap();
        let count = counters.get(phone).copied().unwrap_or(0);
        Ok(count >= self.max_requests)
    }

    async fn increment_sms_counter(&self, phone: &str) -> Result<i64, String> {
        let mut counters = self.phone_counters.lock().unwrap();
        let count = counters.entry(phone.to_string()).or_insert(0);
        *count += 1;
        Ok(*count)
    }

    async fn get_rate_limit_reset_time(&self, _phone: &str) -> Result<Option<i64>, String> {
        Ok(Some(3600))
    }
    
    async fn check_ip_verification_limit(&self, ip: &str) -> Result<bool, String> {
        let counters = self.ip_counters.lock().unwrap();
        let count = counters.get(ip).copied().unwrap_or(0);
        Ok(count >= self.max_ip_attempts)
    }
    
    async fn increment_ip_verification_counter(&self, ip: &str) -> Result<i64, String> {
        let mut counters = self.ip_counters.lock().unwrap();
        let count = counters.entry(ip.to_string()).or_insert(0);
        *count += 1;
        Ok(*count)
    }
    
    async fn get_ip_rate_limit_reset_time(&self, _ip: &str) -> Result<Option<i64>, String> {
        // Return reset time in seconds (1 hour)
        Ok(Some(3600))
    }
    
    async fn log_rate_limit_violation(
        &self, 
        identifier: &str, 
        identifier_type: &str,
        action: &str
    ) -> Result<(), String> {
        let mut logs = self.rate_limit_logs.lock().unwrap();
        logs.push((
            identifier.to_string(),
            identifier_type.to_string(),
            action.to_string()
        ));
        Ok(())
    }
}