//! Tests for authentication error messages in English
//! This test file verifies that all auth-related error messages have proper English translations

#[cfg(test)]
mod tests {
    use re_api::i18n::{get_error_message, format_message, Language};
    use std::collections::HashMap;

    #[test]
    fn test_auth_error_messages_english() {
        // Test all authentication error messages have English translations
        let auth_errors = vec![
            "invalid_phone_format",
            "invalid_chinese_phone",
            "invalid_australian_phone",
            "missing_country_code",
            "rate_limit_exceeded",
            "sms_service_failure",
            "invalid_verification_code",
            "verification_code_expired",
            "max_attempts_exceeded",
            "user_not_found",
            "user_already_exists",
            "authentication_failed",
            "insufficient_permissions",
            "account_suspended",
            "session_expired",
            "registration_disabled",
            "user_blocked",
            "phone_locked",
            "sms_rate_limit_exceeded",
            "account_locked",
            "rate_limit_error",
            "invalid_chinese_phone_format",
            "invalid_australian_phone_format",
            "unsupported_country_code",
            "verification_failed",
            "unknown_error",
        ];

        for error_key in auth_errors {
            let result = get_error_message("auth", error_key, Language::English);
            assert!(
                result.is_some(),
                "Missing English translation for auth.{}",
                error_key
            );
            
            if let Some((code, message, _status)) = result {
                assert!(!message.is_empty(), "Empty English message for auth.{}", error_key);
                assert_eq!(code, error_key, "Code mismatch for auth.{}", error_key);
                
                // Ensure message is in English (doesn't contain Chinese characters)
                assert!(
                    !message.chars().any(|c| c as u32 >= 0x4E00 && c as u32 <= 0x9FFF),
                    "English message contains Chinese characters for auth.{}: {}",
                    error_key,
                    message
                );
            }
        }
    }

    #[test]
    fn test_token_error_messages_english() {
        // Test all token error messages have English translations
        let token_errors = vec![
            "token_expired",
            "invalid_token_format",
            "invalid_signature",
            "token_not_yet_valid",
            "invalid_claims",
            "token_revoked",
            "refresh_token_expired",
            "invalid_refresh_token",
            "token_generation_failed",
            "missing_claim",
            "key_load_error",
        ];

        for error_key in token_errors {
            let result = get_error_message("token", error_key, Language::English);
            assert!(
                result.is_some(),
                "Missing English translation for token.{}",
                error_key
            );
            
            if let Some((code, message, _status)) = result {
                assert!(!message.is_empty(), "Empty English message for token.{}", error_key);
                assert_eq!(code, error_key, "Code mismatch for token.{}", error_key);
                
                // Ensure message is in English (doesn't contain Chinese characters)
                assert!(
                    !message.chars().any(|c| c as u32 >= 0x4E00 && c as u32 <= 0x9FFF),
                    "English message contains Chinese characters for token.{}: {}",
                    error_key,
                    message
                );
            }
        }
    }

    #[test]
    fn test_validation_error_messages_english() {
        // Test all validation error messages have English translations
        let validation_errors = vec![
            "required_field",
            "invalid_format",
            "out_of_range",
            "invalid_length",
            "pattern_mismatch",
            "invalid_email",
            "invalid_url",
            "invalid_date",
            "duplicate_value",
            "business_rule_violation",
        ];

        for error_key in validation_errors {
            let result = get_error_message("validation", error_key, Language::English);
            assert!(
                result.is_some(),
                "Missing English translation for validation.{}",
                error_key
            );
            
            if let Some((code, message, _status)) = result {
                assert!(!message.is_empty(), "Empty English message for validation.{}", error_key);
                assert_eq!(code, error_key, "Code mismatch for validation.{}", error_key);
                
                // Ensure message is in English (doesn't contain Chinese characters)
                assert!(
                    !message.chars().any(|c| c as u32 >= 0x4E00 && c as u32 <= 0x9FFF),
                    "English message contains Chinese characters for validation.{}: {}",
                    error_key,
                    message
                );
            }
        }
    }

    #[test]
    fn test_message_formatting_with_parameters() {
        // Test that message formatting works correctly with parameters
        let test_cases = vec![
            (
                "auth",
                "invalid_phone_format",
                vec![("phone", "+1234567890")],
                "Invalid phone number format. Must include country code (e.g., +86 for China, +61 for Australia): +1234567890",
            ),
            (
                "auth",
                "rate_limit_exceeded",
                vec![("minutes", "5")],
                "Too many requests. Please try again in 5 minutes",
            ),
            (
                "validation",
                "required_field",
                vec![("field", "email")],
                "Required field: email",
            ),
            (
                "validation",
                "out_of_range",
                vec![("field", "age"), ("min", "18"), ("max", "100")],
                "Field age out of range (min: 18, max: 100)",
            ),
            (
                "token",
                "missing_claim",
                vec![("claim", "user_id")],
                "Missing required claim: user_id",
            ),
            (
                "token",
                "key_load_error",
                vec![("message", "File not found")],
                "Failed to load cryptographic key: File not found",
            ),
        ];

        for (category, error_key, params, expected) in test_cases {
            let result = get_error_message(category, error_key, Language::English);
            assert!(result.is_some(), "Missing message for {}.{}", category, error_key);
            
            if let Some((_code, message_template, _status)) = result {
                let mut param_map = HashMap::new();
                for (key, value) in params {
                    param_map.insert(key, value.to_string());
                }
                
                let formatted = format_message(&message_template, &param_map);
                assert_eq!(
                    formatted, expected,
                    "Incorrect formatting for {}.{}",
                    category, error_key
                );
            }
        }
    }

    #[test]
    fn test_error_message_quality_english() {
        // Test that English error messages are user-friendly and professional
        let quality_checks = vec![
            ("auth", "sms_service_failure", vec!["temporarily", "try again"]),
            ("auth", "session_expired", vec!["Please", "login"]),
            ("auth", "max_attempts_exceeded", vec!["Please", "request"]),
            ("auth", "registration_disabled", vec!["currently"]),
            ("token", "token_expired", vec!["expired"]),
            ("validation", "invalid_email", vec!["email", "format"]),
        ];

        for (category, error_key, expected_words) in quality_checks {
            let result = get_error_message(category, error_key, Language::English);
            assert!(result.is_some());
            
            if let Some((_code, message, _status)) = result {
                for word in expected_words {
                    assert!(
                        message.to_lowercase().contains(&word.to_lowercase()),
                        "Message for {}.{} should contain '{}': {}",
                        category,
                        error_key,
                        word,
                        message
                    );
                }
            }
        }
    }

    #[test]
    fn test_http_status_codes_are_appropriate() {
        // Test that HTTP status codes are appropriate for error types
        let status_checks = vec![
            ("auth", "invalid_phone_format", 400),
            ("auth", "rate_limit_exceeded", 429),
            ("auth", "sms_service_failure", 503),
            ("auth", "user_not_found", 404),
            ("auth", "user_already_exists", 409),
            ("auth", "authentication_failed", 401),
            ("auth", "insufficient_permissions", 403),
            ("token", "token_expired", 401),
            ("token", "key_load_error", 500),
            ("validation", "required_field", 400),
            ("validation", "duplicate_value", 409),
            ("general", "internal_error", 500),
            ("general", "service_unavailable", 503),
            ("general", "not_found", 404),
            ("general", "unauthorized", 401),
            ("general", "forbidden", 403),
        ];

        for (category, error_key, expected_status) in status_checks {
            let result = get_error_message(category, error_key, Language::English);
            assert!(result.is_some());
            
            if let Some((_code, _message, status)) = result {
                assert_eq!(
                    status, expected_status,
                    "Incorrect HTTP status for {}.{}",
                    category, error_key
                );
            }
        }
    }
}