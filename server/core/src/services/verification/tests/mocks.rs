//! Mock implementations for testing verification service

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::domain::entities::verification_code::MAX_ATTEMPTS;
use crate::services::verification::traits::{SmsServiceTrait, CacheServiceTrait};

// Mock SMS service for testing
pub struct MockSmsService {
    pub sent_messages: Arc<Mutex<HashMap<String, String>>>,
    pub should_fail: bool,
}

impl MockSmsService {
    pub fn new(should_fail: bool) -> Self {
        Self {
            sent_messages: Arc::new(Mutex::new(HashMap::new())),
            should_fail,
        }
    }

    pub fn get_sent_code(&self, phone: &str) -> Option<String> {
        self.sent_messages.lock().unwrap().get(phone).cloned()
    }
}

#[async_trait]
impl SmsServiceTrait for MockSmsService {
    async fn send_verification_code(&self, phone: &str, code: &str) -> Result<String, String> {
        if self.should_fail {
            return Err("SMS service error".to_string());
        }
        self.sent_messages
            .lock()
            .unwrap()
            .insert(phone.to_string(), code.to_string());
        Ok(format!("mock-msg-{}", uuid::Uuid::new_v4()))
    }

    fn is_valid_phone_number(&self, phone: &str) -> bool {
        phone.starts_with('+') && phone.len() >= 10
    }
}

// Mock cache service for testing
pub struct MockCacheService {
    pub codes: Arc<Mutex<HashMap<String, (String, i32)>>>, // phone -> (code, attempts)
    pub should_fail: bool,
}

impl MockCacheService {
    pub fn new(should_fail: bool) -> Self {
        Self {
            codes: Arc::new(Mutex::new(HashMap::new())),
            should_fail,
        }
    }
}

#[async_trait]
impl CacheServiceTrait for MockCacheService {
    async fn store_code(&self, phone: &str, code: &str) -> Result<(), String> {
        if self.should_fail {
            return Err("Cache service error".to_string());
        }
        self.codes
            .lock()
            .unwrap()
            .insert(phone.to_string(), (code.to_string(), 0));
        Ok(())
    }

    async fn verify_code(&self, phone: &str, code: &str) -> Result<bool, String> {
        if self.should_fail {
            return Err("Cache service error".to_string());
        }
        
        let mut codes = self.codes.lock().unwrap();
        if let Some((stored_code, attempts)) = codes.get_mut(phone) {
            *attempts += 1;
            if *attempts > MAX_ATTEMPTS {
                return Ok(false);
            }
            if stored_code == code {
                codes.remove(phone);
                return Ok(true);
            }
        }
        Ok(false)
    }

    async fn get_remaining_attempts(&self, phone: &str) -> Result<i64, String> {
        if self.should_fail {
            return Err("Cache service error".to_string());
        }
        let codes = self.codes.lock().unwrap();
        if let Some((_, attempts)) = codes.get(phone) {
            Ok((MAX_ATTEMPTS - attempts).max(0) as i64)
        } else {
            Ok(MAX_ATTEMPTS as i64)
        }
    }

    async fn code_exists(&self, phone: &str) -> Result<bool, String> {
        if self.should_fail {
            return Err("Cache service error".to_string());
        }
        Ok(self.codes.lock().unwrap().contains_key(phone))
    }

    async fn get_code_ttl(&self, phone: &str) -> Result<Option<i64>, String> {
        if self.should_fail {
            return Err("Cache service error".to_string());
        }
        if self.codes.lock().unwrap().contains_key(phone) {
            Ok(Some(300)) // Mock 5 minutes
        } else {
            Ok(None)
        }
    }

    async fn clear_verification(&self, phone: &str) -> Result<(), String> {
        if self.should_fail {
            return Err("Cache service error".to_string());
        }
        self.codes.lock().unwrap().remove(phone);
        Ok(())
    }
}