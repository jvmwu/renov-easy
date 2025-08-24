//! Tests for AWS SNS SMS Service

use crate::sms::aws_sns::{AwsSnsConfig, AwsSnsSmsService};
use crate::sms::sms_service::SmsService;

#[test]
fn test_aws_sns_config_from_env() {
    // Clean up any existing env vars first - including alternates
    std::env::remove_var("AWS_ACCESS_KEY_ID");
    std::env::remove_var("AWS_SECRET_ACCESS_KEY");
    std::env::remove_var("AWS_REGION");
    std::env::remove_var("AWS_SNS_ACCESS_KEY_ID");
    std::env::remove_var("AWS_SNS_SECRET_ACCESS_KEY");
    std::env::remove_var("AWS_SNS_REGION");
    std::env::remove_var("AWS_SNS_SENDER_ID");
    std::env::remove_var("AWS_SNS_SMS_TYPE");
    std::env::remove_var("AWS_SNS_MAX_RETRIES");
    std::env::remove_var("AWS_SNS_RETRY_DELAY_MS");
    std::env::remove_var("AWS_SNS_REQUEST_TIMEOUT_SECS");
    
    // Set required environment variables
    std::env::set_var("AWS_ACCESS_KEY_ID", "test_access_key");
    std::env::set_var("AWS_SECRET_ACCESS_KEY", "test_secret_key");
    
    let config = AwsSnsConfig::from_env();
    assert!(config.is_ok());
    
    let config = config.unwrap();
    assert_eq!(config.access_key_id, "test_access_key");
    assert_eq!(config.secret_access_key, "test_secret_key");
    assert_eq!(config.region, "us-east-1"); // Default region
    assert_eq!(config.sms_type, "Transactional"); // Default SMS type
    assert_eq!(config.max_retries, 3); // Default retries
    assert_eq!(config.retry_delay_ms, 1000); // Default delay
    assert_eq!(config.request_timeout_secs, 30); // Default timeout
    
    // Clean up
    std::env::remove_var("AWS_ACCESS_KEY_ID");
    std::env::remove_var("AWS_SECRET_ACCESS_KEY");
}

#[test]
fn test_aws_sns_config_with_custom_values() {
    // Clean up first
    std::env::remove_var("AWS_ACCESS_KEY_ID");
    std::env::remove_var("AWS_SECRET_ACCESS_KEY");
    std::env::remove_var("AWS_REGION");
    
    // Set custom environment variables
    std::env::set_var("AWS_SNS_ACCESS_KEY_ID", "custom_access_key");
    std::env::set_var("AWS_SNS_SECRET_ACCESS_KEY", "custom_secret_key");
    std::env::set_var("AWS_SNS_REGION", "eu-west-1");
    std::env::set_var("AWS_SNS_SENDER_ID", "RenovEasy");
    std::env::set_var("AWS_SNS_SMS_TYPE", "Promotional");
    std::env::set_var("AWS_SNS_MAX_RETRIES", "5");
    std::env::set_var("AWS_SNS_RETRY_DELAY_MS", "2000");
    std::env::set_var("AWS_SNS_REQUEST_TIMEOUT_SECS", "60");
    
    let config = AwsSnsConfig::from_env();
    assert!(config.is_ok());
    
    let config = config.unwrap();
    assert_eq!(config.access_key_id, "custom_access_key");
    assert_eq!(config.secret_access_key, "custom_secret_key");
    assert_eq!(config.region, "eu-west-1");
    assert_eq!(config.sender_id, Some("RenovEasy".to_string()));
    assert_eq!(config.sms_type, "Promotional");
    assert_eq!(config.max_retries, 5);
    assert_eq!(config.retry_delay_ms, 2000);
    assert_eq!(config.request_timeout_secs, 60);
    
    // Clean up
    std::env::remove_var("AWS_SNS_ACCESS_KEY_ID");
    std::env::remove_var("AWS_SNS_SECRET_ACCESS_KEY");
    std::env::remove_var("AWS_SNS_REGION");
    std::env::remove_var("AWS_SNS_SENDER_ID");
    std::env::remove_var("AWS_SNS_SMS_TYPE");
    std::env::remove_var("AWS_SNS_MAX_RETRIES");
    std::env::remove_var("AWS_SNS_RETRY_DELAY_MS");
    std::env::remove_var("AWS_SNS_REQUEST_TIMEOUT_SECS");
}

#[test]
fn test_aws_sns_config_validation() {
    // Clean up first
    std::env::remove_var("AWS_SNS_ACCESS_KEY_ID");
    std::env::remove_var("AWS_SNS_SECRET_ACCESS_KEY");
    
    // Test invalid SMS type
    std::env::set_var("AWS_ACCESS_KEY_ID", "test_key");
    std::env::set_var("AWS_SECRET_ACCESS_KEY", "test_secret");
    std::env::set_var("AWS_SNS_SMS_TYPE", "Invalid");
    
    let config = AwsSnsConfig::from_env();
    assert!(config.is_err());
    assert!(config.unwrap_err().to_string().contains("'Transactional' or 'Promotional'"));
    
    // Clean up
    std::env::remove_var("AWS_ACCESS_KEY_ID");
    std::env::remove_var("AWS_SECRET_ACCESS_KEY");
    std::env::remove_var("AWS_SNS_SMS_TYPE");
}

#[test]
fn test_aws_sns_config_missing_credentials() {
    // Clean environment
    std::env::remove_var("AWS_ACCESS_KEY_ID");
    std::env::remove_var("AWS_SECRET_ACCESS_KEY");
    std::env::remove_var("AWS_SNS_ACCESS_KEY_ID");
    std::env::remove_var("AWS_SNS_SECRET_ACCESS_KEY");
    
    let config = AwsSnsConfig::from_env();
    assert!(config.is_err());
    assert!(config.unwrap_err().to_string().contains("AWS_ACCESS_KEY_ID"));
}

#[tokio::test]
async fn test_aws_sns_service_creation() {
    // Note: This test will fail in CI without actual AWS credentials
    // It's mainly for local testing with real credentials
    
    let config = AwsSnsConfig {
        access_key_id: "test_key".to_string(),
        secret_access_key: "test_secret".to_string(),
        region: "us-east-1".to_string(),
        sender_id: Some("Test".to_string()),
        sms_type: "Transactional".to_string(),
        max_retries: 3,
        retry_delay_ms: 1000,
        request_timeout_secs: 30,
    };
    
    // This will create a client but won't actually connect to AWS
    let service = AwsSnsSmsService::new(config).await;
    assert!(service.is_ok());
    
    let service = service.unwrap();
    assert_eq!(service.provider_name(), "AWS SNS");
}

#[tokio::test]
async fn test_phone_number_validation() {
    // Test various phone number formats
    let test_cases = vec![
        ("+14155552671", true, Some("+14155552671")),
        ("+919876543210", true, Some("+919876543210")),
        ("+33612345678", true, Some("+33612345678")),
        ("4155552671", true, Some("+14155552671")), // US number without country code
        ("1234567890", true, Some("+11234567890")), // Assumed US
        ("+1", false, None), // Too short
        ("+", false, None), // Just the plus sign
        ("", false, None), // Empty string
    ];
    
    // Create a service with test configuration
    let config = AwsSnsConfig {
        access_key_id: "test_key".to_string(),
        secret_access_key: "test_secret".to_string(),
        region: "us-east-1".to_string(),
        sender_id: None,
        sms_type: "Transactional".to_string(),
        max_retries: 3,
        retry_delay_ms: 1000,
        request_timeout_secs: 30,
    };
    
    // Create service (this will create a real AWS client, but won't make API calls)
    let service = AwsSnsSmsService::new(config).await.unwrap();
    
    for (input, should_succeed, expected) in test_cases {
        let result = service.validate_phone_number(input);
        
        if should_succeed {
            assert!(result.is_ok(), "Failed to validate: {}", input);
            if let Some(expected_value) = expected {
                assert_eq!(result.unwrap(), expected_value, "Unexpected result for: {}", input);
            }
        } else {
            assert!(result.is_err(), "Should have failed validation: {}", input);
        }
    }
}