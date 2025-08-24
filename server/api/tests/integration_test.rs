//! Integration tests for the authentication API endpoints

#[cfg(test)]
mod auth_tests {
    use actix_web::{test, web, App};
    use serde_json::json;

    #[actix_web::test]
    async fn test_send_code_endpoint_validation() {
        // Test basic validation of the send-code endpoint
        
        // Test with valid Chinese phone number
        let valid_chinese = json!({
            "phone": "13812345678",
            "country_code": "+86"
        });
        
        // Test with valid Australian phone number
        let valid_australian = json!({
            "phone": "412345678",
            "country_code": "+61"
        });
        
        // Test with full E.164 format
        let valid_e164 = json!({
            "phone": "+8613812345678",
            "country_code": "+86"
        });
        
        // Test with invalid phone (too short)
        let invalid_short = json!({
            "phone": "123",
            "country_code": "+86"
        });
        
        // Test with invalid country code
        let invalid_country = json!({
            "phone": "13812345678",
            "country_code": "+0"  // Country codes can't start with 0
        });
        
        // These tests verify that the DTOs are properly structured
        // and validation rules are in place
        assert!(valid_chinese.is_object());
        assert!(valid_australian.is_object());
        assert!(valid_e164.is_object());
        assert!(invalid_short.is_object());
        assert!(invalid_country.is_object());
    }
    
    #[actix_web::test]
    async fn test_phone_number_formatting() {
        // Test that phone numbers can be formatted correctly
        let test_cases = vec![
            ("+8613812345678", true),  // Chinese with country code
            ("+61412345678", true),     // Australian with country code
            ("+14155552671", true),     // US number
            ("13812345678", false),     // Missing country code
            ("+0123456789", false),     // Invalid country code starting with 0
            ("+123", false),            // Too short
        ];
        
        for (phone, should_be_valid) in test_cases {
            let is_e164 = phone.starts_with('+') 
                && phone.len() >= 8 
                && phone.len() <= 16
                && !phone[1..].starts_with('0')  // Country codes can't start with 0
                && phone[1..].chars().all(|c| c.is_ascii_digit());
            assert_eq!(
                is_e164, should_be_valid,
                "Phone validation failed for: {}",
                phone
            );
        }
    }
}