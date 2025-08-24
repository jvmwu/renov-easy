//! Unit tests for token entities

use uuid::Uuid;
use chrono::{Duration, Utc};
use crate::domain::entities::token::{
    Claims, RefreshToken, TokenPair,
    ACCESS_TOKEN_EXPIRY_MINUTES, REFRESH_TOKEN_EXPIRY_DAYS,
    JWT_ISSUER, JWT_AUDIENCE
};

#[test]
fn test_access_token_claims() {
    let user_id = Uuid::new_v4();
    let claims = Claims::new_access_token(
        user_id,
        Some("customer".to_string()),
        true,
        Some("phone_hash_123".to_string()),
        Some("device_fingerprint_456".to_string()),
    );
    
    assert_eq!(claims.sub, user_id.to_string());
    assert_eq!(claims.iss, JWT_ISSUER);
    assert_eq!(claims.aud, JWT_AUDIENCE);
    assert_eq!(claims.user_type, Some("customer".to_string()));
    assert!(claims.is_verified);
    assert_eq!(claims.phone_hash, Some("phone_hash_123".to_string()));
    assert_eq!(claims.device_fingerprint, Some("device_fingerprint_456".to_string()));
    assert!(claims.is_valid());
    assert!(!claims.is_expired());
}

#[test]
fn test_refresh_token_claims() {
    let user_id = Uuid::new_v4();
    let token_family = Some("family_123".to_string());
    let device_fingerprint = Some("device_789".to_string());
    let claims = Claims::new_refresh_token(user_id, token_family.clone(), device_fingerprint.clone());
    
    assert_eq!(claims.sub, user_id.to_string());
    assert_eq!(claims.iss, JWT_ISSUER);
    assert_eq!(claims.aud, JWT_AUDIENCE);
    assert_eq!(claims.user_type, None);
    assert!(!claims.is_verified);
    assert_eq!(claims.token_family, token_family);
    assert_eq!(claims.device_fingerprint, device_fingerprint);
    assert!(claims.is_valid());
    assert!(!claims.is_expired());
}

#[test]
fn test_claims_user_id_parsing() {
    let user_id = Uuid::new_v4();
    let claims = Claims::new_access_token(user_id, None, false, None, None);
    
    let parsed_id = claims.user_id().unwrap();
    assert_eq!(parsed_id, user_id);
}

#[test]
fn test_claims_expiration() {
    let user_id = Uuid::new_v4();
    let mut claims = Claims::new_access_token(user_id, None, false, None, None);
    
    // Set expiration to past
    claims.exp = Utc::now().timestamp() - 1;
    
    assert!(claims.is_expired());
    assert!(!claims.is_valid());
}

#[test]
fn test_claims_not_before() {
    let user_id = Uuid::new_v4();
    let mut claims = Claims::new_access_token(user_id, None, false, None, None);
    
    // Set nbf to future
    claims.nbf = Utc::now().timestamp() + 3600;
    
    assert!(!claims.is_valid());
}

#[test]
fn test_refresh_token_creation() {
    let user_id = Uuid::new_v4();
    let token_hash = "hashed_token_value".to_string();
    let token = RefreshToken::new(user_id, token_hash.clone());
    
    assert_eq!(token.user_id, user_id);
    assert_eq!(token.token_hash, token_hash);
    assert!(!token.is_revoked);
    assert!(!token.is_expired());
    assert!(token.is_valid());
}

#[test]
fn test_refresh_token_revocation() {
    let user_id = Uuid::new_v4();
    let mut token = RefreshToken::new(user_id, "hash".to_string());
    
    assert!(token.is_valid());
    
    token.revoke();
    
    assert!(token.is_revoked);
    assert!(!token.is_valid());
}

#[test]
fn test_refresh_token_expiration() {
    let user_id = Uuid::new_v4();
    let mut token = RefreshToken::new(user_id, "hash".to_string());
    
    // Manually set expiration to past
    token.expires_at = Utc::now() - Duration::days(1);
    
    assert!(token.is_expired());
    assert!(!token.is_valid());
}

#[test]
fn test_refresh_token_time_until_expiration() {
    let user_id = Uuid::new_v4();
    let token = RefreshToken::new(user_id, "hash".to_string());
    
    let time_remaining = token.time_until_expiration();
    let expected_max = Duration::days(REFRESH_TOKEN_EXPIRY_DAYS);
    let expected_min = Duration::days(REFRESH_TOKEN_EXPIRY_DAYS - 1);
    
    assert!(time_remaining <= expected_max);
    assert!(time_remaining > expected_min);
}

#[test]
fn test_token_pair_creation() {
    let access = "access_token_jwt".to_string();
    let refresh = "refresh_token_jwt".to_string();
    let pair = TokenPair::new(access.clone(), refresh.clone());
    
    assert_eq!(pair.access_token, access);
    assert_eq!(pair.refresh_token, refresh);
    assert_eq!(pair.access_expires_in, ACCESS_TOKEN_EXPIRY_MINUTES * 60);
    assert_eq!(pair.refresh_expires_in, REFRESH_TOKEN_EXPIRY_DAYS * 24 * 60 * 60);
}

#[test]
fn test_token_pair_serialization() {
    let pair = TokenPair::new(
        "access_token".to_string(),
        "refresh_token".to_string(),
    );
    
    // Serialize to JSON
    let json = serde_json::to_string(&pair).unwrap();
    
    // Deserialize back
    let deserialized: TokenPair = serde_json::from_str(&json).unwrap();
    
    assert_eq!(pair, deserialized);
}

#[test]
fn test_claims_serialization() {
    let user_id = Uuid::new_v4();
    let claims = Claims::new_access_token(
        user_id,
        Some("worker".to_string()),
        true,
        Some("phone_hash_test".to_string()),
        Some("device_test".to_string()),
    );
    
    // Serialize to JSON
    let json = serde_json::to_string(&claims).unwrap();
    
    // Deserialize back
    let deserialized: Claims = serde_json::from_str(&json).unwrap();
    
    assert_eq!(claims, deserialized);
}

#[test]
fn test_refresh_token_serialization() {
    let user_id = Uuid::new_v4();
    let token = RefreshToken::new(user_id, "token_hash".to_string());
    
    // Serialize to JSON
    let json = serde_json::to_string(&token).unwrap();
    
    // Deserialize back
    let deserialized: RefreshToken = serde_json::from_str(&json).unwrap();
    
    assert_eq!(token, deserialized);
}

#[test]
fn test_refresh_token_with_metadata() {
    let user_id = Uuid::new_v4();
    let token_hash = "hashed_token".to_string();
    let token_family = Some("family_abc".to_string());
    let device_fingerprint = Some("device_xyz".to_string());
    let previous_token_id = Some(Uuid::new_v4());
    
    let token = RefreshToken::new_with_metadata(
        user_id,
        token_hash.clone(),
        token_family.clone(),
        device_fingerprint.clone(),
        previous_token_id,
    );
    
    assert_eq!(token.user_id, user_id);
    assert_eq!(token.token_hash, token_hash);
    assert_eq!(token.token_family, token_family);
    assert_eq!(token.device_fingerprint, device_fingerprint);
    assert_eq!(token.previous_token_id, previous_token_id);
    assert!(!token.is_revoked);
    assert!(token.is_valid());
}

#[test]
fn test_token_pair_with_metadata() {
    let access = "access_token".to_string();
    let refresh = "refresh_token".to_string();
    let token_family = Some("family_123".to_string());
    let device_fingerprint = Some("device_456".to_string());
    
    let pair = TokenPair::new_with_metadata(
        access.clone(),
        refresh.clone(),
        token_family.clone(),
        device_fingerprint.clone(),
    );
    
    assert_eq!(pair.access_token, access);
    assert_eq!(pair.refresh_token, refresh);
    assert_eq!(pair.token_family, token_family);
    assert_eq!(pair.device_fingerprint, device_fingerprint);
    assert_eq!(pair.token_type, "Bearer");
    assert!(pair.issued_at > 0);
}

#[test]
fn test_claims_with_phone_hash() {
    let user_id = Uuid::new_v4();
    let phone_hash = Some("phone_hash_sha256".to_string());
    
    let claims = Claims::new_access_token(
        user_id,
        Some("customer".to_string()),
        true,
        phone_hash.clone(),
        None,
    );
    
    assert_eq!(claims.phone_hash, phone_hash);
    assert_eq!(claims.device_fingerprint, None);
    assert_eq!(claims.token_family, None);
}