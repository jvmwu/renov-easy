//! Integration tests for the OTP encryption service module

use re_core::services::encryption::{
    AesGcmOtpEncryption, OtpEncryption, OtpEncryptionConfig,
    KeyManager, KeyRotationConfig,
};

#[test]
fn test_otp_encryption_integration() {
    // Create encryption service
    let config = OtpEncryptionConfig::default();
    let service = AesGcmOtpEncryption::new(config).expect("Failed to create encryption service");
    
    // Test data
    let otp = "123456";
    let phone = "+1234567890";
    let expiration_minutes = 5;
    
    // Encrypt OTP
    let encrypted = service
        .encrypt_otp(otp, phone, expiration_minutes)
        .expect("Failed to encrypt OTP");
    
    // Verify encrypted fields
    assert!(!encrypted.ciphertext.is_empty());
    assert!(!encrypted.nonce.is_empty());
    assert!(!encrypted.key_id.is_empty());
    assert_eq!(encrypted.phone, phone);
    assert_eq!(encrypted.attempt_count, 0);
    
    // Decrypt and verify
    let decrypted = service
        .decrypt_otp(&encrypted)
        .expect("Failed to decrypt OTP");
    assert_eq!(decrypted, otp);
    
    // Verify OTP with correct code
    let is_valid = service
        .verify_otp(&encrypted, otp)
        .expect("Failed to verify OTP");
    assert!(is_valid);
    
    // Verify OTP with wrong code
    let is_valid = service
        .verify_otp(&encrypted, "654321")
        .expect("Failed to verify OTP");
    assert!(!is_valid);
}

#[test]
fn test_key_rotation() {
    // Create key manager
    let config = KeyRotationConfig {
        max_key_age_days: 30,
        keep_old_keys: true,
        max_old_keys: 3,
    };
    let manager = KeyManager::new(config).expect("Failed to create key manager");
    
    // Get initial key
    let initial_key = manager.get_active_key().expect("Failed to get active key");
    assert!(initial_key.is_active);
    
    // Rotate key
    let new_key_id = manager.rotate_key().expect("Failed to rotate key");
    assert_ne!(initial_key.id, new_key_id);
    
    // Verify new key is active
    let new_key = manager.get_active_key().expect("Failed to get new active key");
    assert_eq!(new_key.id, new_key_id);
    assert!(new_key.is_active);
    
    // Verify old key is still accessible
    let old_key = manager.get_key(&initial_key.id).expect("Failed to get old key");
    assert!(!old_key.is_active);
}

#[test]
fn test_encryption_with_key_rotation() {
    let config = OtpEncryptionConfig::default();
    let service = AesGcmOtpEncryption::new(config).expect("Failed to create encryption service");
    
    let otp1 = "111111";
    let otp2 = "222222";
    let phone = "+9876543210";
    
    // Encrypt first OTP
    let encrypted1 = service
        .encrypt_otp(otp1, phone, 5)
        .expect("Failed to encrypt first OTP");
    
    // Rotate key
    service.rotate_key().expect("Failed to rotate key");
    
    // Encrypt second OTP with new key
    let encrypted2 = service
        .encrypt_otp(otp2, phone, 5)
        .expect("Failed to encrypt second OTP");
    
    // Keys should be different
    assert_ne!(encrypted1.key_id, encrypted2.key_id);
    
    // Both should still be decryptable
    let decrypted1 = service
        .decrypt_otp(&encrypted1)
        .expect("Failed to decrypt first OTP");
    assert_eq!(decrypted1, otp1);
    
    let decrypted2 = service
        .decrypt_otp(&encrypted2)
        .expect("Failed to decrypt second OTP");
    assert_eq!(decrypted2, otp2);
}

#[test]
fn test_constant_time_verification() {
    let config = OtpEncryptionConfig::default();
    let service = AesGcmOtpEncryption::new(config).expect("Failed to create encryption service");
    
    let otp = "123456";
    let phone = "+5555555555";
    
    let encrypted = service
        .encrypt_otp(otp, phone, 5)
        .expect("Failed to encrypt OTP");
    
    // Test various incorrect codes
    let wrong_codes = vec![
        "000000",  // Completely different
        "123455",  // Last digit different
        "223456",  // First digit different
        "123457",  // One off
        "12345",   // Too short
        "1234567", // Too long
    ];
    
    for wrong_code in wrong_codes {
        let is_valid = service
            .verify_otp(&encrypted, wrong_code)
            .expect("Failed to verify OTP");
        assert!(!is_valid, "Wrong code {} should not be valid", wrong_code);
    }
    
    // Correct code should still work
    let is_valid = service
        .verify_otp(&encrypted, otp)
        .expect("Failed to verify OTP");
    assert!(is_valid);
}

#[test]
fn test_expired_otp() {
    use chrono::Utc;
    
    let config = OtpEncryptionConfig::default();
    let service = AesGcmOtpEncryption::new(config).expect("Failed to create encryption service");
    
    let otp = "999999";
    let phone = "+1111111111";
    
    // Create an OTP that expires immediately
    let mut encrypted = service
        .encrypt_otp(otp, phone, 0)
        .expect("Failed to encrypt OTP");
    
    // Manually set expiration to past
    encrypted.expires_at = Utc::now() - chrono::Duration::minutes(1);
    
    // Verification should fail for expired OTP
    let is_valid = service
        .verify_otp(&encrypted, otp)
        .expect("Failed to verify OTP");
    assert!(!is_valid, "Expired OTP should not be valid");
}