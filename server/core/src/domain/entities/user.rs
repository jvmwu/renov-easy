//! User entity representing a registered user in the RenovEasy system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents the type of user in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserType {
    /// A customer seeking renovation services
    Customer,
    /// A worker providing renovation services
    Worker,
}

/// User entity representing a registered user
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    /// Unique identifier for the user
    pub id: Uuid,
    
    /// Hashed phone number for security
    #[serde(rename = "phone_hash")]
    pub phone_hash: String,
    
    /// Country code (e.g., +86, +61)
    pub country_code: String,
    
    /// Type of user (Customer or Worker)
    pub user_type: Option<UserType>,
    
    /// Timestamp when the user was created
    pub created_at: DateTime<Utc>,
    
    /// Timestamp when the user was last updated
    pub updated_at: DateTime<Utc>,
    
    /// Timestamp of the user's last login
    pub last_login_at: Option<DateTime<Utc>>,
    
    /// Whether the user's phone number has been verified
    pub is_verified: bool,
    
    /// Whether the user account is blocked
    pub is_blocked: bool,
}

impl User {
    /// Creates a new User instance
    pub fn new(
        phone_hash: String,
        country_code: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            phone_hash,
            country_code,
            user_type: None,
            created_at: now,
            updated_at: now,
            last_login_at: None,
            is_verified: false,
            is_blocked: false,
        }
    }
    
    /// Sets the user type
    pub fn set_user_type(&mut self, user_type: UserType) {
        self.user_type = Some(user_type);
        self.updated_at = Utc::now();
    }
    
    /// Marks the user as verified
    pub fn verify(&mut self) {
        self.is_verified = true;
        self.updated_at = Utc::now();
    }
    
    /// Blocks the user account
    pub fn block(&mut self) {
        self.is_blocked = true;
        self.updated_at = Utc::now();
    }
    
    /// Unblocks the user account
    pub fn unblock(&mut self) {
        self.is_blocked = false;
        self.updated_at = Utc::now();
    }
    
    /// Updates the last login timestamp
    pub fn update_last_login(&mut self) {
        self.last_login_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
    
    /// Checks if the user has selected a user type
    pub fn has_user_type(&self) -> bool {
        self.user_type.is_some()
    }
    
    /// Checks if the user is a customer
    pub fn is_customer(&self) -> bool {
        matches!(self.user_type, Some(UserType::Customer))
    }
    
    /// Checks if the user is a worker
    pub fn is_worker(&self) -> bool {
        matches!(self.user_type, Some(UserType::Worker))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new_user_creation() {
        let user = User::new(
            "hashed_phone_123".to_string(),
            "+61".to_string(),
        );
        
        assert_eq!(user.phone_hash, "hashed_phone_123");
        assert_eq!(user.country_code, "+61");
        assert_eq!(user.user_type, None);
        assert!(!user.is_verified);
        assert!(!user.is_blocked);
        assert!(user.last_login_at.is_none());
    }
    
    #[test]
    fn test_set_user_type() {
        let mut user = User::new(
            "hashed_phone".to_string(),
            "+86".to_string(),
        );
        
        user.set_user_type(UserType::Customer);
        assert_eq!(user.user_type, Some(UserType::Customer));
        assert!(user.is_customer());
        assert!(!user.is_worker());
    }
    
    #[test]
    fn test_user_verification() {
        let mut user = User::new(
            "hashed_phone".to_string(),
            "+61".to_string(),
        );
        
        assert!(!user.is_verified);
        user.verify();
        assert!(user.is_verified);
    }
    
    #[test]
    fn test_user_blocking() {
        let mut user = User::new(
            "hashed_phone".to_string(),
            "+86".to_string(),
        );
        
        assert!(!user.is_blocked);
        user.block();
        assert!(user.is_blocked);
        user.unblock();
        assert!(!user.is_blocked);
    }
    
    #[test]
    fn test_update_last_login() {
        let mut user = User::new(
            "hashed_phone".to_string(),
            "+61".to_string(),
        );
        
        assert!(user.last_login_at.is_none());
        user.update_last_login();
        assert!(user.last_login_at.is_some());
    }
    
    #[test]
    fn test_user_type_serialization() {
        let customer = UserType::Customer;
        let json = serde_json::to_string(&customer).unwrap();
        assert_eq!(json, "\"customer\"");
        
        let worker = UserType::Worker;
        let json = serde_json::to_string(&worker).unwrap();
        assert_eq!(json, "\"worker\"");
    }
}