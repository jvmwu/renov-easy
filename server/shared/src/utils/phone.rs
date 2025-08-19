//! Phone number utilities

use regex::Regex;
use once_cell::sync::Lazy;

// Chinese mobile phone number regex
static CHINA_MOBILE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^1[3-9]\d{9}$").unwrap()
});

// International phone number regex (E.164 format)
static INTERNATIONAL_PHONE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\+[1-9]\d{1,14}$").unwrap()
});

/// Normalize a phone number by removing common formatting characters
pub fn normalize_phone_number(phone: &str) -> String {
    phone
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '+')
        .collect()
}

/// Check if a phone number is valid (Chinese mobile)
pub fn is_valid_chinese_mobile(phone: &str) -> bool {
    let normalized = normalize_phone_number(phone);
    CHINA_MOBILE_REGEX.is_match(&normalized)
}

/// Check if a phone number is valid (international E.164 format)
pub fn is_valid_international_phone(phone: &str) -> bool {
    let normalized = normalize_phone_number(phone);
    INTERNATIONAL_PHONE_REGEX.is_match(&normalized)
}

/// Check if a phone number is valid (either Chinese or international)
pub fn is_valid_phone(phone: &str) -> bool {
    let normalized = normalize_phone_number(phone);
    is_valid_chinese_mobile(&normalized) || is_valid_international_phone(&normalized)
}

/// Format a Chinese mobile number for display
pub fn format_chinese_mobile(phone: &str) -> Option<String> {
    let normalized = normalize_phone_number(phone);
    if is_valid_chinese_mobile(&normalized) {
        Some(format!(
            "{} {} {}",
            &normalized[0..3],
            &normalized[3..7],
            &normalized[7..11]
        ))
    } else {
        None
    }
}

/// Mask a phone number for display (e.g., 138****5678)
pub fn mask_phone_number(phone: &str) -> String {
    let normalized = normalize_phone_number(phone);
    if normalized.len() >= 7 {
        format!(
            "{}****{}",
            &normalized[0..3],
            &normalized[normalized.len() - 4..]
        )
    } else {
        "****".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_phone_number() {
        assert_eq!(normalize_phone_number("138-1234-5678"), "13812345678");
        assert_eq!(normalize_phone_number("+86 138 1234 5678"), "+8613812345678");
        assert_eq!(normalize_phone_number("(138) 1234-5678"), "13812345678");
    }

    #[test]
    fn test_is_valid_chinese_mobile() {
        assert!(is_valid_chinese_mobile("13812345678"));
        assert!(is_valid_chinese_mobile("15912345678"));
        assert!(is_valid_chinese_mobile("18612345678"));
        assert!(!is_valid_chinese_mobile("12812345678")); // Invalid prefix
        assert!(!is_valid_chinese_mobile("1381234567"));   // Too short
        assert!(!is_valid_chinese_mobile("138123456789")); // Too long
    }

    #[test]
    fn test_is_valid_international_phone() {
        assert!(is_valid_international_phone("+8613812345678"));
        assert!(is_valid_international_phone("+14155552671"));
        assert!(is_valid_international_phone("+442071838750"));
        assert!(!is_valid_international_phone("13812345678")); // Missing +
        assert!(!is_valid_international_phone("+0123456789"));  // Invalid country code
    }

    #[test]
    fn test_format_chinese_mobile() {
        assert_eq!(
            format_chinese_mobile("13812345678"),
            Some("138 1234 5678".to_string())
        );
        assert_eq!(format_chinese_mobile("invalid"), None);
    }

    #[test]
    fn test_mask_phone_number() {
        assert_eq!(mask_phone_number("13812345678"), "138****5678");
        assert_eq!(mask_phone_number("+8613812345678"), "+86****5678");
        assert_eq!(mask_phone_number("12345"), "****");
    }
}