//! Verification code entity for SMS-based authentication.

use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Maximum number of verification attempts allowed
pub const MAX_ATTEMPTS: i32 = 3;

/// Length of the verification code
pub const CODE_LENGTH: usize = 6;

/// Default expiration time for verification codes (5 minutes)
pub const DEFAULT_EXPIRATION_MINUTES: i64 = 5;

/// Verification code entity for SMS-based authentication
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationCode {
    /// Unique identifier for the verification code
    pub id: Uuid,
    
    /// Phone number this code was sent to (with country code)
    pub phone: String,
    
    /// The 6-digit verification code
    pub code: String,
    
    /// Number of verification attempts made
    pub attempts: i32,
    
    /// Timestamp when the code was created
    pub created_at: DateTime<Utc>,
    
    /// Timestamp when the code expires
    pub expires_at: DateTime<Utc>,
    
    /// Whether the code has been successfully used
    pub is_used: bool,
}

impl VerificationCode {
    /// Creates a new verification code with a cryptographically secure random 6-digit code
    ///
    /// # Arguments
    ///
    /// * `phone` - The phone number to send the code to (with country code)
    ///
    /// # Returns
    ///
    /// A new `VerificationCode` instance with a random 6-digit code
    pub fn new(phone: String) -> Self {
        let code = Self::generate_code();
        let now = Utc::now();
        let expires_at = now + Duration::minutes(DEFAULT_EXPIRATION_MINUTES);
        
        Self {
            id: Uuid::new_v4(),
            phone,
            code,
            attempts: 0,
            created_at: now,
            expires_at,
            is_used: false,
        }
    }
    
    /// Creates a new verification code with a custom expiration time
    ///
    /// # Arguments
    ///
    /// * `phone` - The phone number to send the code to (with country code)
    /// * `expiration_minutes` - Number of minutes until the code expires
    ///
    /// # Returns
    ///
    /// A new `VerificationCode` instance with custom expiration
    pub fn new_with_expiration(phone: String, expiration_minutes: i64) -> Self {
        let code = Self::generate_code();
        let now = Utc::now();
        let expires_at = now + Duration::minutes(expiration_minutes);
        
        Self {
            id: Uuid::new_v4(),
            phone,
            code,
            attempts: 0,
            created_at: now,
            expires_at,
            is_used: false,
        }
    }
    
    /// Generates a cryptographically secure random 6-digit code
    ///
    /// # Returns
    ///
    /// A string containing a 6-digit verification code
    fn generate_code() -> String {
        let mut rng = rand::thread_rng();
        let code: u32 = rng.gen_range(0..1_000_000);
        format!("{:06}", code)
    }
    
    /// Checks if the verification code has expired
    ///
    /// # Returns
    ///
    /// `true` if the code has expired, `false` otherwise
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
    
    /// Checks if the verification code is still valid
    ///
    /// A code is valid if:
    /// - It hasn't expired
    /// - It hasn't been used
    /// - The maximum number of attempts hasn't been exceeded
    ///
    /// # Returns
    ///
    /// `true` if the code is valid, `false` otherwise
    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.is_used && self.attempts < MAX_ATTEMPTS
    }
    
    /// Verifies if the provided code matches this verification code
    ///
    /// This method increments the attempt counter if the code doesn't match
    /// and marks the code as used if it matches.
    ///
    /// # Arguments
    ///
    /// * `input_code` - The code to verify
    ///
    /// # Returns
    ///
    /// `Ok(())` if the code matches and is valid, `Err` with an appropriate message otherwise
    pub fn verify(&mut self, input_code: &str) -> Result<(), String> {
        // Check if the code is expired
        if self.is_expired() {
            return Err("Verification code has expired".to_string());
        }
        
        // Check if the code has already been used
        if self.is_used {
            return Err("Verification code has already been used".to_string());
        }
        
        // Check if maximum attempts exceeded
        if self.attempts >= MAX_ATTEMPTS {
            return Err("Maximum verification attempts exceeded".to_string());
        }
        
        // Increment attempts
        self.attempts += 1;
        
        // Verify the code
        if self.code == input_code {
            self.is_used = true;
            Ok(())
        } else {
            let remaining = MAX_ATTEMPTS - self.attempts;
            if remaining > 0 {
                Err(format!(
                    "Invalid verification code. {} attempt(s) remaining",
                    remaining
                ))
            } else {
                Err("Invalid verification code. No attempts remaining".to_string())
            }
        }
    }
    
    /// Gets the number of remaining verification attempts
    ///
    /// # Returns
    ///
    /// The number of remaining attempts (0 if exceeded)
    pub fn remaining_attempts(&self) -> i32 {
        (MAX_ATTEMPTS - self.attempts).max(0)
    }
    
    /// Gets the time remaining until expiration
    ///
    /// # Returns
    ///
    /// A `Duration` representing the time until expiration, or zero if expired
    pub fn time_until_expiration(&self) -> Duration {
        let now = Utc::now();
        if self.expires_at > now {
            self.expires_at - now
        } else {
            Duration::zero()
        }
    }
    
    /// Marks the verification code as used
    pub fn mark_as_used(&mut self) {
        self.is_used = true;
    }
    
    /// Resets the verification code with a new random code
    ///
    /// This is useful for resending a new code to the same phone number
    pub fn reset(&mut self) {
        self.code = Self::generate_code();
        self.attempts = 0;
        self.is_used = false;
        self.created_at = Utc::now();
        self.expires_at = self.created_at + Duration::minutes(DEFAULT_EXPIRATION_MINUTES);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration as StdDuration;
    
    #[test]
    fn test_new_verification_code() {
        let phone = "+61412345678".to_string();
        let code = VerificationCode::new(phone.clone());
        
        assert_eq!(code.phone, phone);
        assert_eq!(code.code.len(), CODE_LENGTH);
        assert_eq!(code.attempts, 0);
        assert!(!code.is_used);
        assert!(!code.is_expired());
        assert!(code.is_valid());
    }
    
    #[test]
    fn test_generate_code_format() {
        // Test multiple times to ensure consistency
        for _ in 0..100 {
            let code = VerificationCode::generate_code();
            assert_eq!(code.len(), CODE_LENGTH);
            assert!(code.chars().all(|c| c.is_ascii_digit()));
            
            // Verify it's a valid number
            let num: u32 = code.parse().expect("Generated code should be a valid number");
            assert!(num < 1_000_000);
        }
    }
    
    #[test]
    fn test_code_uniqueness() {
        // Generate multiple codes and check they're not all the same
        let codes: Vec<String> = (0..100)
            .map(|_| VerificationCode::generate_code())
            .collect();
        
        // There should be at least some unique codes (extremely unlikely to get all same)
        let unique_count = codes.iter().collect::<std::collections::HashSet<_>>().len();
        assert!(unique_count > 1);
    }
    
    #[test]
    fn test_verification_success() {
        let mut code = VerificationCode::new("+61412345678".to_string());
        let verification_code = code.code.clone();
        
        let result = code.verify(&verification_code);
        assert!(result.is_ok());
        assert!(code.is_used);
        assert_eq!(code.attempts, 1);
    }
    
    #[test]
    fn test_verification_failure() {
        let mut code = VerificationCode::new("+61412345678".to_string());
        
        let result = code.verify("000000");
        assert!(result.is_err());
        assert!(!code.is_used);
        assert_eq!(code.attempts, 1);
        assert_eq!(code.remaining_attempts(), 2);
    }
    
    #[test]
    fn test_max_attempts() {
        let mut code = VerificationCode::new("+61412345678".to_string());
        let correct_code = code.code.clone();
        
        // Make MAX_ATTEMPTS with wrong code
        for i in 1..=MAX_ATTEMPTS {
            let result = code.verify("000000");
            assert!(result.is_err());
            assert_eq!(code.attempts, i);
        }
        
        // Next attempt should fail due to max attempts
        let result = code.verify(&correct_code);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Maximum verification attempts exceeded"));
    }
    
    #[test]
    fn test_already_used_code() {
        let mut code = VerificationCode::new("+61412345678".to_string());
        let verification_code = code.code.clone();
        
        // First verification should succeed
        assert!(code.verify(&verification_code).is_ok());
        
        // Second verification should fail because code is already used
        let result = code.verify(&verification_code);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already been used"));
    }
    
    #[test]
    fn test_custom_expiration() {
        let phone = "+61412345678".to_string();
        let expiration_minutes = 10;
        let code = VerificationCode::new_with_expiration(phone, expiration_minutes);
        
        let expected_expiration = code.created_at + Duration::minutes(expiration_minutes);
        assert_eq!(code.expires_at, expected_expiration);
    }
    
    #[test]
    fn test_is_expired() {
        // Create a code that expires immediately (0 minutes)
        let mut code = VerificationCode::new_with_expiration("+61412345678".to_string(), 0);
        let verification_code = code.code.clone();
        
        // Sleep for a short time to ensure expiration
        thread::sleep(StdDuration::from_millis(10));
        
        assert!(code.is_expired());
        assert!(!code.is_valid());
        
        // Verification should fail due to expiration
        let result = code.verify(&verification_code);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expired"));
    }
    
    #[test]
    fn test_reset_code() {
        let mut code = VerificationCode::new("+61412345678".to_string());
        let original_code = code.code.clone();
        
        // Make some attempts
        code.verify("000000").ok();
        code.verify("111111").ok();
        assert_eq!(code.attempts, 2);
        
        // Reset the code
        code.reset();
        
        // Check that everything is reset
        assert_ne!(code.code, original_code); // Should have a new code
        assert_eq!(code.attempts, 0);
        assert!(!code.is_used);
        assert!(code.is_valid());
    }
    
    #[test]
    fn test_remaining_attempts() {
        let mut code = VerificationCode::new("+61412345678".to_string());
        
        assert_eq!(code.remaining_attempts(), MAX_ATTEMPTS);
        
        code.verify("000000").ok();
        assert_eq!(code.remaining_attempts(), MAX_ATTEMPTS - 1);
        
        code.verify("111111").ok();
        assert_eq!(code.remaining_attempts(), MAX_ATTEMPTS - 2);
        
        code.verify("222222").ok();
        assert_eq!(code.remaining_attempts(), 0);
    }
    
    #[test]
    fn test_time_until_expiration() {
        let code = VerificationCode::new("+61412345678".to_string());
        
        let time_remaining = code.time_until_expiration();
        assert!(time_remaining <= Duration::minutes(DEFAULT_EXPIRATION_MINUTES));
        assert!(time_remaining > Duration::minutes(DEFAULT_EXPIRATION_MINUTES - 1));
    }
    
    #[test]
    fn test_serialization() {
        let code = VerificationCode::new("+61412345678".to_string());
        
        // Serialize to JSON
        let json = serde_json::to_string(&code).unwrap();
        
        // Deserialize back
        let deserialized: VerificationCode = serde_json::from_str(&json).unwrap();
        
        assert_eq!(code, deserialized);
    }
}