//! Unit tests for SMS service creation

use crate::sms::create_sms_service;
use crate::config::SmsConfig;

#[tokio::test]
async fn test_create_mock_service() {
    let config = SmsConfig {
        provider: "mock".to_string(),
        api_key: String::new(),
        api_secret: String::new(),
        from_number: "+1234567890".to_string(),
    };

    let service = create_sms_service(&config).await;
    assert_eq!(service.provider_name(), "Mock");
}

#[tokio::test]
async fn test_create_unknown_provider_fallback() {
    let config = SmsConfig {
        provider: "unknown".to_string(),
        api_key: String::new(),
        api_secret: String::new(),
        from_number: "+1234567890".to_string(),
    };

    let service = create_sms_service(&config).await;
    // Should fallback to mock
    assert_eq!(service.provider_name(), "Mock");
}