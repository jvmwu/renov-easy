//! Phone number utility functions for authentication service

use sha2::{Sha256, Digest};

/// Validates if a phone number is in valid E.164 format
///
/// # Arguments
///
/// * `phone` - Phone number to validate
///
/// # Returns
///
/// * `bool` - True if valid, false otherwise
pub fn is_valid_phone_format(phone: &str) -> bool {
    // Check basic E.164 format requirements
    if !phone.starts_with('+') {
        return false;
    }

    // Must be between 10 and 15 digits after the +
    let digits = &phone[1..];
    if digits.len() < 10 || digits.len() > 15 {
        return false;
    }

    // All characters after + must be digits
    if !digits.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    true
}

/// Mask phone number for logging (show only last 4 digits)
///
/// # Arguments
///
/// * `phone` - Phone number to mask
///
/// # Returns
///
/// * `String` - Masked phone number
pub fn mask_phone(phone: &str) -> String {
    if phone.len() <= 4 {
        return "*".repeat(phone.len());
    }
    format!("***{}", &phone[phone.len() - 4..])
}

/// Hash a phone number using SHA-256
///
/// # Arguments
///
/// * `phone` - Phone number to hash (without country code)
///
/// # Returns
///
/// * `String` - Hexadecimal representation of SHA-256 hash
pub fn hash_phone(phone: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(phone.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Extract country code from a full phone number
///
/// # Arguments
///
/// * `phone` - Full phone number in E.164 format (e.g., +1234567890)
///
/// # Returns
///
/// * `(String, String)` - Tuple of (country_code, phone_without_country_code)
///
/// # Note
///
/// This is a simple implementation that handles common country codes.
/// For production, consider using a proper phone number parsing library.
pub fn extract_country_code(phone: &str) -> (String, String) {
    // Common country codes by length
    // 1 digit: +1 (US/Canada), +7 (Russia)
    // 2 digits: +86 (China), +61 (Australia), +44 (UK), etc.
    // 3 digits: +358 (Finland), +972 (Israel), etc.
    
    // Try common patterns
    if phone.starts_with("+1") && phone.len() == 11 {
        // US/Canada
        ("+1".to_string(), phone[2..].to_string())
    } else if phone.starts_with("+86") {
        // China
        ("+86".to_string(), phone[3..].to_string())
    } else if phone.starts_with("+61") {
        // Australia
        ("+61".to_string(), phone[3..].to_string())
    } else if phone.starts_with("+44") {
        // UK
        ("+44".to_string(), phone[3..].to_string())
    } else if phone.starts_with("+7") && phone.len() == 12 {
        // Russia
        ("+7".to_string(), phone[2..].to_string())
    } else {
        // Default: assume 2-digit country code for now
        // This is a simplification and should be improved with a proper library
        if phone.len() > 3 && phone[1..3].chars().all(|c| c.is_ascii_digit()) {
            (phone[0..3].to_string(), phone[3..].to_string())
        } else {
            // Fallback to single digit
            (phone[0..2].to_string(), phone[2..].to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_phone() {
        assert_eq!(mask_phone("+1234567890"), "***7890");
        assert_eq!(mask_phone("+123"), "****");
        assert_eq!(mask_phone("1234"), "****");
        assert_eq!(mask_phone("123"), "***");
    }
    
    #[test]
    fn test_hash_phone() {
        let phone = "1234567890";
        let hash = hash_phone(phone);
        // SHA-256 hash should be 64 characters long (hex representation)
        assert_eq!(hash.len(), 64);
        // Should be consistent
        let hash2 = hash_phone(phone);
        assert_eq!(hash, hash2);
        // Different input should produce different hash
        let hash3 = hash_phone("0987654321");
        assert_ne!(hash, hash3);
    }
    
    #[test]
    fn test_extract_country_code() {
        // US/Canada
        assert_eq!(
            extract_country_code("+1234567890"),
            ("+1".to_string(), "234567890".to_string())
        );
        // China
        assert_eq!(
            extract_country_code("+8613812345678"),
            ("+86".to_string(), "13812345678".to_string())
        );
        // Australia
        assert_eq!(
            extract_country_code("+61412345678"),
            ("+61".to_string(), "412345678".to_string())
        );
        // UK
        assert_eq!(
            extract_country_code("+447123456789"),
            ("+44".to_string(), "7123456789".to_string())
        );
        // Russia
        assert_eq!(
            extract_country_code("+79123456789"),
            ("+7".to_string(), "9123456789".to_string())
        );
    }

    #[test]
    fn test_is_valid_phone_format() {
        // Valid formats
        assert!(is_valid_phone_format("+1234567890"));
        assert!(is_valid_phone_format("+861234567890"));
        assert!(is_valid_phone_format("+12345678901234"));

        // Invalid formats
        assert!(!is_valid_phone_format("1234567890")); // Missing +
        assert!(!is_valid_phone_format("+123")); // Too short
        assert!(!is_valid_phone_format("+1234567890123456")); // Too long
        assert!(!is_valid_phone_format("+123abc7890")); // Contains letters
        assert!(!is_valid_phone_format("")); // Empty
    }
}