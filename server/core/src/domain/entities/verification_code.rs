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
    #[cfg(test)]
    pub fn generate_code() -> String {
        let mut rng = rand::thread_rng();
        let code: u32 = rng.gen_range(0..1_000_000);
        format!("{:06}", code)
    }
    
    #[cfg(not(test))]
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