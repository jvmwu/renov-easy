//! Unit tests for SMS service

use crate::sms::{mask_phone_number, is_valid_phone_number};

#[test]
fn test_mask_phone_number() {
    assert_eq!(mask_phone_number("+1234567890"), "+******7890");
    assert_eq!(mask_phone_number("+12345678901234"), "+**********1234");
    assert_eq!(mask_phone_number("1234567890"), "******7890");
    assert_eq!(mask_phone_number("123"), "***");
    assert_eq!(mask_phone_number("1234"), "****");
}

#[test]
fn test_is_valid_phone_number() {
    // Valid numbers
    assert!(is_valid_phone_number("+1234567890"));
    assert!(is_valid_phone_number("+12345678901"));
    assert!(is_valid_phone_number("+123456789012345"));
    
    // Invalid numbers
    assert!(!is_valid_phone_number("1234567890")); // No plus
    assert!(!is_valid_phone_number("+123")); // Too short
    assert!(!is_valid_phone_number("+1234567890123456")); // Too long
    assert!(!is_valid_phone_number("+123abc4567")); // Contains letters
    assert!(!is_valid_phone_number("+")); // Only plus sign
}