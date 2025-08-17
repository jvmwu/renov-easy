//! Unit tests for user entity

use crate::domain::entities::user::{User, UserType};

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

#[test]
fn test_has_user_type() {
    let mut user = User::new(
        "hashed_phone".to_string(),
        "+61".to_string(),
    );
    
    assert!(!user.has_user_type());
    user.set_user_type(UserType::Worker);
    assert!(user.has_user_type());
}

#[test]
fn test_is_customer() {
    let mut user = User::new(
        "hashed_phone".to_string(),
        "+61".to_string(),
    );
    
    assert!(!user.is_customer());
    user.set_user_type(UserType::Customer);
    assert!(user.is_customer());
    assert!(!user.is_worker());
}

#[test]
fn test_is_worker() {
    let mut user = User::new(
        "hashed_phone".to_string(),
        "+61".to_string(),
    );
    
    assert!(!user.is_worker());
    user.set_user_type(UserType::Worker);
    assert!(user.is_worker());
    assert!(!user.is_customer());
}

#[test]
fn test_user_serialization() {
    let user = User::new(
        "hashed_phone".to_string(),
        "+61".to_string(),
    );
    
    // Serialize to JSON
    let json = serde_json::to_string(&user).unwrap();
    
    // Deserialize back
    let deserialized: User = serde_json::from_str(&json).unwrap();
    
    assert_eq!(user, deserialized);
}

#[test]
fn test_user_with_type_serialization() {
    let mut user = User::new(
        "hashed_phone".to_string(),
        "+86".to_string(),
    );
    user.set_user_type(UserType::Customer);
    user.verify();
    
    // Serialize to JSON
    let json = serde_json::to_string(&user).unwrap();
    
    // Deserialize back
    let deserialized: User = serde_json::from_str(&json).unwrap();
    
    assert_eq!(user, deserialized);
    assert!(deserialized.is_customer());
    assert!(deserialized.is_verified);
}