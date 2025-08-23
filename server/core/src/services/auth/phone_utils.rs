//! Phone number utility functions for authentication service
//!
//! This module provides comprehensive phone number validation and manipulation
//! utilities supporting E.164 format and country-specific validation rules.

use sha2::{Sha256, Digest};
use once_cell::sync::Lazy;
use regex::Regex;

/// Regular expression for valid E.164 format
/// E.164 format: + followed by 1-3 digit country code (no leading 0) and up to 14 total digits
static E164_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\+[1-9]\d{6,14}$").unwrap()
});

/// Regular expression for Chinese mobile numbers (without country code)
static CHINA_MOBILE_REGEX: Lazy<Regex> = Lazy::new(|| {
    // Chinese mobile numbers start with 13-19, followed by 9 digits
    Regex::new(r"^1[3-9]\d{9}$").unwrap()
});

/// Regular expression for Australian mobile numbers (without country code)
static AUSTRALIA_MOBILE_REGEX: Lazy<Regex> = Lazy::new(|| {
    // Australian mobile numbers start with 4, followed by 8 digits
    Regex::new(r"^4\d{8}$").unwrap()
});

/// Supported country codes with their validation rules
#[derive(Debug, Clone, PartialEq)]
pub enum CountryCode {
    China,      // +86
    Australia,  // +61
    US,         // +1
    Canada,     // +1
    UK,         // +44
    Russia,     // +7
    Other(String),
}

impl CountryCode {
    /// Parse country code from the beginning of a phone number
    pub fn from_phone(phone: &str) -> Option<(Self, &str)> {
        if !phone.starts_with('+') {
            return None;
        }
        
        // Check common country codes
        if phone.starts_with("+86") {
            Some((CountryCode::China, &phone[3..]))
        } else if phone.starts_with("+61") {
            Some((CountryCode::Australia, &phone[3..]))
        } else if phone.starts_with("+1") && phone.len() == 12 {
            // North American Numbering Plan (US/Canada) - 11 digits total
            Some((CountryCode::US, &phone[2..]))
        } else if phone.starts_with("+44") {
            Some((CountryCode::UK, &phone[3..]))
        } else if phone.starts_with("+7") && phone.len() == 12 {
            Some((CountryCode::Russia, &phone[2..]))
        } else {
            // Try to extract a generic country code (1-3 digits)
            let digits = phone[1..].chars().take_while(|c| c.is_ascii_digit()).count();
            if digits >= 1 && digits <= 3 {
                let code = phone[0..=digits].to_string();
                let remaining = &phone[digits + 1..];
                Some((CountryCode::Other(code), remaining))
            } else {
                None
            }
        }
    }
    
    /// Get the country code string
    pub fn as_str(&self) -> &str {
        match self {
            CountryCode::China => "+86",
            CountryCode::Australia => "+61",
            CountryCode::US | CountryCode::Canada => "+1",
            CountryCode::UK => "+44",
            CountryCode::Russia => "+7",
            CountryCode::Other(code) => code,
        }
    }
}

/// Validates if a phone number is in valid E.164 format
///
/// E.164 format requirements:
/// - Starts with '+'
/// - Country code (1-3 digits, cannot start with 0)
/// - Total length including '+' is 8-16 characters
/// - Only digits after '+'
///
/// # Arguments
///
/// * `phone` - Phone number to validate
///
/// # Returns
///
/// * `bool` - True if valid E.164 format, false otherwise
///
/// # Examples
///
/// ```
/// assert!(is_valid_phone_format("+8613812345678")); // China
/// assert!(is_valid_phone_format("+61412345678"));   // Australia
/// assert!(is_valid_phone_format("+14155552671"));   // US
/// assert!(!is_valid_phone_format("13812345678"));   // Missing +
/// ```
pub fn is_valid_phone_format(phone: &str) -> bool {
    E164_REGEX.is_match(phone)
}

/// Validates a Chinese phone number
///
/// Supports:
/// - Full E.164 format with +86 prefix
/// - Local format (11 digits starting with 13-19)
///
/// # Arguments
///
/// * `phone` - Phone number to validate
///
/// # Returns
///
/// * `bool` - True if valid Chinese phone number
///
/// # Examples
///
/// ```
/// assert!(validate_chinese_phone("+8613812345678"));
/// assert!(validate_chinese_phone("13812345678"));
/// assert!(!validate_chinese_phone("+8612812345678")); // Invalid prefix
/// ```
pub fn validate_chinese_phone(phone: &str) -> bool {
    if phone.starts_with("+86") {
        // Full international format
        let local_number = &phone[3..];
        CHINA_MOBILE_REGEX.is_match(local_number)
    } else if phone.starts_with("+") {
        // Has country code but not Chinese
        false
    } else {
        // Local format
        CHINA_MOBILE_REGEX.is_match(phone)
    }
}

/// Validates an Australian phone number
///
/// Supports:
/// - Full E.164 format with +61 prefix
/// - Local format (9 digits starting with 4)
///
/// # Arguments
///
/// * `phone` - Phone number to validate
///
/// # Returns
///
/// * `bool` - True if valid Australian phone number
///
/// # Examples
///
/// ```
/// assert!(validate_australian_phone("+61412345678"));
/// assert!(validate_australian_phone("0412345678")); // With leading 0
/// assert!(validate_australian_phone("412345678"));  // Without leading 0
/// ```
pub fn validate_australian_phone(phone: &str) -> bool {
    if phone.starts_with("+61") {
        // Full international format
        let local_number = &phone[3..];
        // Remove leading 0 if present (common in Australian format)
        let normalized = if local_number.starts_with('0') {
            &local_number[1..]
        } else {
            local_number
        };
        AUSTRALIA_MOBILE_REGEX.is_match(normalized)
    } else if phone.starts_with("+") {
        // Has country code but not Australian
        false
    } else {
        // Local format - handle with or without leading 0
        let normalized = if phone.starts_with('0') && phone.len() == 10 {
            &phone[1..]
        } else {
            phone
        };
        AUSTRALIA_MOBILE_REGEX.is_match(normalized)
    }
}

/// Validates phone number with country-specific rules
///
/// This function provides comprehensive validation by:
/// 1. First checking E.164 format compliance
/// 2. Then applying country-specific validation rules
///
/// # Arguments
///
/// * `phone` - Phone number to validate
///
/// # Returns
///
/// * `bool` - True if valid according to country-specific rules
pub fn validate_phone_with_country(phone: &str) -> bool {
    // First check E.164 format
    if !is_valid_phone_format(phone) {
        return false;
    }
    
    // Apply country-specific validation
    if let Some((country, _)) = CountryCode::from_phone(phone) {
        match country {
            CountryCode::China => validate_chinese_phone(phone),
            CountryCode::Australia => validate_australian_phone(phone),
            _ => true, // For other countries, E.164 validation is sufficient
        }
    } else {
        false
    }
}

/// Normalize phone number to E.164 format
///
/// Converts local phone numbers to E.164 format based on country rules:
/// - Chinese numbers without +86 get it added
/// - Australian numbers without +61 get it added (removes leading 0)
///
/// # Arguments
///
/// * `phone` - Phone number to normalize
/// * `default_country` - Default country to assume if no country code present
///
/// # Returns
///
/// * `Option<String>` - Normalized E.164 phone number, or None if invalid
pub fn normalize_to_e164(phone: &str, default_country: Option<CountryCode>) -> Option<String> {
    // Remove common formatting characters
    let cleaned: String = phone.chars()
        .filter(|c| c.is_ascii_digit() || *c == '+')
        .collect();
    
    // If already in E.164 format, validate and return
    if cleaned.starts_with('+') {
        if is_valid_phone_format(&cleaned) {
            return Some(cleaned);
        } else {
            return None;
        }
    }
    
    // Apply default country code if provided
    match default_country {
        Some(CountryCode::China) => {
            if CHINA_MOBILE_REGEX.is_match(&cleaned) {
                Some(format!("+86{}", cleaned))
            } else {
                None
            }
        },
        Some(CountryCode::Australia) => {
            let normalized = if cleaned.starts_with('0') && cleaned.len() == 10 {
                &cleaned[1..]
            } else {
                &cleaned
            };
            if AUSTRALIA_MOBILE_REGEX.is_match(normalized) {
                Some(format!("+61{}", normalized))
            } else {
                None
            }
        },
        _ => None,
    }
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
/// # Examples
///
/// ```
/// let (code, local) = extract_country_code("+8613812345678");
/// assert_eq!(code, "+86");
/// assert_eq!(local, "13812345678");
/// ```
pub fn extract_country_code(phone: &str) -> (String, String) {
    if let Some((country, local)) = CountryCode::from_phone(phone) {
        (country.as_str().to_string(), local.to_string())
    } else {
        // Fallback for invalid format
        if phone.starts_with('+') && phone.len() > 2 {
            (phone[0..2].to_string(), phone[2..].to_string())
        } else {
            (String::new(), phone.to_string())
        }
    }
}

/// Get a descriptive error message for invalid phone numbers
///
/// Provides specific error messages based on the validation failure:
/// - Missing country code
/// - Invalid format for specific country
/// - Invalid E.164 format
///
/// # Arguments
///
/// * `phone` - The invalid phone number
/// * `expected_country` - Optional expected country for more specific errors
///
/// # Returns
///
/// * `(String, String)` - Tuple of (English message, Chinese message)
pub fn get_validation_error(phone: &str, expected_country: Option<CountryCode>) -> (String, String) {
    if !phone.starts_with('+') {
        (
            "Phone number must include country code (e.g., +86 for China, +61 for Australia)".to_string(),
            "电话号码必须包含国家代码（例如：中国 +86，澳大利亚 +61）".to_string()
        )
    } else if let Some(country) = expected_country {
        match country {
            CountryCode::China => (
                format!("Invalid Chinese phone number. Must be 11 digits starting with 13-19 after +86"),
                format!("无效的中国手机号码。+86 后必须是以 13-19 开头的 11 位数字")
            ),
            CountryCode::Australia => (
                format!("Invalid Australian phone number. Must be 9 digits starting with 4 after +61"),
                format!("无效的澳大利亚手机号码。+61 后必须是以 4 开头的 9 位数字")
            ),
            _ => (
                format!("Invalid phone number format for the specified country"),
                format!("指定国家的电话号码格式无效")
            ),
        }
    } else if !is_valid_phone_format(phone) {
        (
            "Invalid phone number format. Must be in E.164 format (e.g., +8613812345678)".to_string(),
            "无效的电话号码格式。必须是 E.164 格式（例如：+8613812345678）".to_string()
        )
    } else {
        (
            "Invalid phone number".to_string(),
            "无效的电话号码".to_string()
        )
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
        // Valid E.164 formats
        assert!(is_valid_phone_format("+1234567890"));
        assert!(is_valid_phone_format("+8613812345678"));
        assert!(is_valid_phone_format("+61412345678"));
        assert!(is_valid_phone_format("+442071234567"));
        assert!(is_valid_phone_format("+12345678901234"));

        // Invalid formats
        assert!(!is_valid_phone_format("1234567890")); // Missing +
        assert!(!is_valid_phone_format("+123")); // Too short
        assert!(!is_valid_phone_format("+1234567890123456")); // Too long (>15 digits)
        assert!(!is_valid_phone_format("+123abc7890")); // Contains letters
        assert!(!is_valid_phone_format("+0123456789")); // Country code starts with 0
        assert!(!is_valid_phone_format("")); // Empty
        assert!(!is_valid_phone_format("+")); // Only plus sign
    }
    
    #[test]
    fn test_validate_chinese_phone() {
        // Valid Chinese phone numbers
        assert!(validate_chinese_phone("+8613812345678"));
        assert!(validate_chinese_phone("+8615912345678"));
        assert!(validate_chinese_phone("+8618612345678"));
        assert!(validate_chinese_phone("+8619012345678"));
        assert!(validate_chinese_phone("13812345678")); // Local format
        assert!(validate_chinese_phone("15912345678"));
        assert!(validate_chinese_phone("18612345678"));
        
        // Invalid Chinese phone numbers
        assert!(!validate_chinese_phone("+8612812345678")); // Invalid prefix 128
        assert!(!validate_chinese_phone("+8610812345678")); // Invalid prefix 108
        assert!(!validate_chinese_phone("+861381234567"));  // Too short
        assert!(!validate_chinese_phone("+86138123456789")); // Too long
        assert!(!validate_chinese_phone("12812345678")); // Invalid local prefix
        assert!(!validate_chinese_phone("+6113812345678")); // Wrong country code
    }
    
    #[test]
    fn test_validate_australian_phone() {
        // Valid Australian phone numbers
        assert!(validate_australian_phone("+61412345678"));
        assert!(validate_australian_phone("+61423456789"));
        assert!(validate_australian_phone("+61456789012"));
        assert!(validate_australian_phone("+61487654321"));
        assert!(validate_australian_phone("0412345678")); // Local with leading 0
        assert!(validate_australian_phone("412345678"));  // Local without leading 0
        
        // Invalid Australian phone numbers
        assert!(!validate_australian_phone("+61312345678")); // Invalid prefix 3
        assert!(!validate_australian_phone("+61512345678")); // Invalid prefix 5
        assert!(!validate_australian_phone("+6141234567"));  // Too short
        assert!(!validate_australian_phone("+614123456789")); // Too long
        assert!(!validate_australian_phone("312345678")); // Invalid local prefix
        assert!(!validate_australian_phone("+86412345678")); // Wrong country code
    }
    
    #[test]
    fn test_validate_phone_with_country() {
        // Chinese phones
        assert!(validate_phone_with_country("+8613812345678"));
        assert!(validate_phone_with_country("+8615912345678"));
        assert!(!validate_phone_with_country("+8612812345678")); // Invalid prefix
        
        // Australian phones
        assert!(validate_phone_with_country("+61412345678"));
        assert!(validate_phone_with_country("+61423456789"));
        assert!(!validate_phone_with_country("+61312345678")); // Invalid prefix
        
        // Other countries (only E.164 validation)
        assert!(validate_phone_with_country("+14155552671")); // US
        assert!(validate_phone_with_country("+442071234567")); // UK
        
        // Invalid E.164 format
        assert!(!validate_phone_with_country("13812345678")); // Missing +
        assert!(!validate_phone_with_country("+0123456789")); // Country code starts with 0
    }
    
    #[test]
    fn test_normalize_to_e164() {
        // Chinese normalization
        assert_eq!(
            normalize_to_e164("13812345678", Some(CountryCode::China)),
            Some("+8613812345678".to_string())
        );
        assert_eq!(
            normalize_to_e164("138-1234-5678", Some(CountryCode::China)),
            Some("+8613812345678".to_string())
        );
        assert_eq!(
            normalize_to_e164("+8613812345678", None),
            Some("+8613812345678".to_string())
        );
        
        // Australian normalization
        assert_eq!(
            normalize_to_e164("0412345678", Some(CountryCode::Australia)),
            Some("+61412345678".to_string())
        );
        assert_eq!(
            normalize_to_e164("412345678", Some(CountryCode::Australia)),
            Some("+61412345678".to_string())
        );
        assert_eq!(
            normalize_to_e164("0412-345-678", Some(CountryCode::Australia)),
            Some("+61412345678".to_string())
        );
        
        // Invalid numbers
        assert_eq!(normalize_to_e164("12812345678", Some(CountryCode::China)), None);
        assert_eq!(normalize_to_e164("312345678", Some(CountryCode::Australia)), None);
        assert_eq!(normalize_to_e164("+123", None), None); // Invalid E.164
    }
    
    #[test]
    fn test_country_code_parsing() {
        // Test CountryCode::from_phone
        assert_eq!(
            CountryCode::from_phone("+8613812345678"),
            Some((CountryCode::China, "13812345678"))
        );
        assert_eq!(
            CountryCode::from_phone("+61412345678"),
            Some((CountryCode::Australia, "412345678"))
        );
        assert_eq!(
            CountryCode::from_phone("+14155552671"),
            Some((CountryCode::US, "4155552671"))
        );
        assert_eq!(
            CountryCode::from_phone("+442071234567"),
            Some((CountryCode::UK, "2071234567"))
        );
        
        // Test as_str
        assert_eq!(CountryCode::China.as_str(), "+86");
        assert_eq!(CountryCode::Australia.as_str(), "+61");
        assert_eq!(CountryCode::US.as_str(), "+1");
        assert_eq!(CountryCode::UK.as_str(), "+44");
    }
    
    #[test]
    fn test_get_validation_error() {
        // Missing country code
        let (en, zh) = get_validation_error("13812345678", None);
        assert!(en.contains("country code"));
        assert!(zh.contains("国家代码"));
        
        // Invalid Chinese phone
        let (en, zh) = get_validation_error("+8612812345678", Some(CountryCode::China));
        assert!(en.contains("Chinese"));
        assert!(zh.contains("中国"));
        
        // Invalid Australian phone
        let (en, zh) = get_validation_error("+61312345678", Some(CountryCode::Australia));
        assert!(en.contains("Australian"));
        assert!(zh.contains("澳大利亚"));
        
        // Invalid E.164 format
        let (en, zh) = get_validation_error("+123", None);
        assert!(en.contains("E.164"));
        assert!(zh.contains("E.164"));
    }
}