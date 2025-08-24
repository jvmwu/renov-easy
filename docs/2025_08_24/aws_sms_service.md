# AWS SNS SMS Service Implementation

## Overview

Implemented AWS SNS SMS service provider as a backup SMS delivery system with automatic failover from Twilio. This provides resilience and redundancy for the passwordless authentication system.

## Components Implemented

### 1. AWS SNS SMS Service (`server/infra/src/sms/aws_sns.rs`)
- Full implementation of the `SmsService` trait using AWS SDK for Rust
- E.164 phone number validation and normalization
- Automatic retry logic with exponential backoff
- Rate limiting handling for AWS API
- Comprehensive error handling
- Phone number masking in logs for security
- Support for SMS sender ID (where supported by region)
- Configurable SMS type (Transactional/Promotional)

### 2. AWS SNS Trait Adapter (`server/infra/src/sms/aws_sns_trait_adapter.rs`)
- Adapter implementing the core `SmsServiceTrait`
- Bridges infrastructure implementation with core domain
- Consistent interface with other SMS providers

### 3. Failover SMS Service (`server/infra/src/sms/failover_sms.rs`)
- Automatic failover from primary (Twilio) to backup (AWS SNS)
- Configurable failover timeout (default: 30 seconds)
- Health check monitoring for automatic recovery
- State tracking for failover events
- Comprehensive logging of failover events

### 4. Configuration Updates (`server/api/src/config.rs`)
- Added AWS SNS configuration support
- Environment variable mapping for AWS credentials
- Support for failover provider configuration
- Validation for production environments

## Environment Variables

### AWS SNS Configuration
```bash
# Primary AWS credentials
AWS_ACCESS_KEY_ID=your_access_key
AWS_SECRET_ACCESS_KEY=your_secret_key
AWS_REGION=us-east-1

# Alternative AWS SNS specific credentials
AWS_SNS_ACCESS_KEY_ID=your_access_key
AWS_SNS_SECRET_ACCESS_KEY=your_secret_key
AWS_SNS_REGION=us-east-1

# Optional settings
AWS_SNS_SENDER_ID=YourApp          # Sender ID (not supported in all regions)
AWS_SNS_SMS_TYPE=Transactional     # or "Promotional"
AWS_SNS_MAX_RETRIES=3               # Default: 3
AWS_SNS_RETRY_DELAY_MS=1000        # Default: 1000ms
AWS_SNS_REQUEST_TIMEOUT_SECS=30    # Default: 30 seconds
```

### SMS Provider Selection
```bash
# Use specific provider
SMS_PROVIDER=aws-sns    # Options: mock, twilio, aws-sns, failover

# Use failover with both Twilio and AWS SNS
SMS_PROVIDER=failover
```

## Usage Examples

### 1. Direct AWS SNS Usage
```rust
use re_infra::sms::{AwsSnsSmsService, AwsSnsConfig};

let config = AwsSnsConfig::from_env()?;
let service = AwsSnsSmsService::new(config).await?;

// Send SMS
let message_id = service.send_sms("+1234567890", "Your code is 123456").await?;
```

### 2. Failover Service Usage
```rust
use re_infra::sms::create_failover_sms_service;

// Automatically creates failover service with Twilio primary and AWS SNS backup
let service = create_failover_sms_service().await;

// Service automatically handles failover
let message_id = service.send_sms("+1234567890", "Your code is 123456").await?;
```

### 3. Using with Core Domain Trait
```rust
use re_infra::sms::AwsSnsSmsServiceAdapter;
use re_core::services::verification::SmsServiceTrait;

let adapter = AwsSnsSmsServiceAdapter::from_env().await?;
let message_id = adapter.send_verification_code("+1234567890", "123456").await?;
```

## Features

### Reliability
- **Automatic Retry**: Implements exponential backoff for transient failures
- **Rate Limiting**: Handles AWS API rate limits gracefully
- **Failover**: Automatic switching from Twilio to AWS SNS on failure
- **Recovery**: Automatic recovery to primary service after timeout

### Security
- **Phone Number Masking**: Sensitive data masked in logs
- **E.164 Validation**: Strict phone number format validation
- **Credential Management**: Support for multiple credential sources

### Performance
- **Async/Await**: Non-blocking I/O operations
- **Connection Pooling**: Efficient AWS SDK client usage
- **Timeout Control**: Configurable request timeouts

## Testing

### Unit Tests
```bash
# Run AWS SNS specific tests
cargo test --package re_infra --features aws-sns --lib sms::tests::aws_sns_tests

# Run with single thread (for env var tests)
cargo test --package re_infra --features aws-sns --lib sms::tests::aws_sns_tests -- --test-threads=1
```

### Integration Tests
```bash
# Run failover integration tests
cargo test --package re_infra --features "twilio-sms aws-sns" --test sms_failover_integration
```

## Requirements Fulfilled

### Requirement 2.10: SMS Provider Failover
✅ **Implemented**: Automatic failover from Twilio to AWS SNS
- Primary service failure detection
- Automatic switching within 30 seconds
- Health check monitoring
- Automatic recovery when primary service is restored

### Technical Requirements
✅ Implements SmsServiceTrait from server/core/src/services/verification/traits.rs
✅ Uses aws-sdk-sns crate for AWS API interaction
✅ Proper error handling and retry logic
✅ Support for sending SMS with OTP codes
✅ AWS credentials configuration (Access Key ID, Secret Access Key, Region)
✅ Phone number validation for international formats
✅ Comprehensive logging for SMS events
✅ Rate limiting handling from AWS SNS API
✅ Monitoring for SMS delivery status
✅ Failover mechanism from Twilio to AWS SNS

## Production Considerations

### AWS SNS Setup
1. Create IAM user with SNS SMS permissions
2. Configure SMS spending limits in AWS console
3. Register sender IDs for supported regions
4. Monitor SMS delivery metrics in CloudWatch

### Cost Management
- AWS SNS charges per SMS sent
- Set spending limits to prevent unexpected costs
- Monitor usage through AWS Cost Explorer

### Regional Considerations
- Not all AWS regions support SMS
- Sender ID support varies by country
- Some countries require pre-registration

## Future Enhancements

1. **Delivery Status Tracking**: Implement webhook handlers for delivery receipts
2. **Cost Optimization**: Dynamic provider selection based on destination and cost
3. **Analytics**: Track success rates and latency per provider
4. **Circuit Breaker**: Implement circuit breaker pattern for provider health
5. **Multiple Backup Providers**: Support for more than one backup provider

## Conclusion

Successfully implemented AWS SNS SMS service provider with automatic failover capability, providing a robust and resilient SMS delivery system for passwordless authentication. The implementation follows all project conventions, includes comprehensive error handling, and is fully tested.