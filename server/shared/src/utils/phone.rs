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
