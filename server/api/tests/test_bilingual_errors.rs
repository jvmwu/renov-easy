use api::handlers::error::{Language, handle_domain_error_with_lang};
use core::errors::{DomainError, AuthError};
use actix_web::http::StatusCode;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_english_error_message() {
        // Create an auth error
        let error = DomainError::Auth(AuthError::InvalidVerificationCode);
        
        // Generate response with English language preference
        let response = handle_domain_error_with_lang(error, Language::English);
        
        // Check status code
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        
        // Parse response body
        let body = response.body();
        if let actix_web::body::MessageBody::Bytes(bytes) = body {
            let json: Value = serde_json::from_slice(bytes.as_ref()).unwrap();
            
            // Check error message is in English
            assert_eq!(json["error"], "invalid_verification_code");
            assert_eq!(json["message"], "Invalid or expired verification code");
        }
    }

    #[test]
    fn test_chinese_error_message() {
        // Create an auth error
        let error = DomainError::Auth(AuthError::InvalidVerificationCode);
        
        // Generate response with Chinese language preference
        let response = handle_domain_error_with_lang(error, Language::Chinese);
        
        // Check status code
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        
        // Parse response body
        let body = response.body();
        if let actix_web::body::MessageBody::Bytes(bytes) = body {
            let json: Value = serde_json::from_slice(bytes.as_ref()).unwrap();
            
            // Check error message is in Chinese
            assert_eq!(json["error"], "invalid_verification_code");
            assert_eq!(json["message"], "验证码无效或已过期");
        }
    }
    
    #[test]
    fn test_rate_limit_error_bilingual() {
        // Test English rate limit error
        let error = DomainError::Auth(AuthError::RateLimitExceeded { minutes: 5 });
        let response = handle_domain_error_with_lang(error, Language::English);
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        
        // Test Chinese rate limit error
        let error = DomainError::Auth(AuthError::RateLimitExceeded { minutes: 5 });
        let response = handle_domain_error_with_lang(error, Language::Chinese);
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }
    
    #[test]
    fn test_language_detection_from_header() {
        use actix_web::test::TestRequest;
        
        // Test Chinese language detection
        let req = TestRequest::default()
            .insert_header(("Accept-Language", "zh-CN,zh;q=0.9,en-US;q=0.8,en;q=0.7"))
            .to_http_request();
        let lang = Language::from_request(&req);
        assert_eq!(lang, Language::Chinese);
        
        // Test English language detection
        let req = TestRequest::default()
            .insert_header(("Accept-Language", "en-US,en;q=0.9,zh-CN;q=0.8"))
            .to_http_request();
        let lang = Language::from_request(&req);
        assert_eq!(lang, Language::English);
        
        // Test default to English when no header
        let req = TestRequest::default().to_http_request();
        let lang = Language::from_request(&req);
        assert_eq!(lang, Language::English);
    }
}