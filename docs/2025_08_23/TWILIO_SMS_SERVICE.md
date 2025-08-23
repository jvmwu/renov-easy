# Twilio SMS Service Implementation

## Overview

The Twilio SMS service has been implemented as the primary SMS provider for the RenovEasy authentication system. This implementation follows the clean architecture pattern and provides reliable SMS delivery with automatic retry logic and comprehensive error handling.

## Features

### Core Capabilities
- **International SMS Support**: Full E.164 format validation and support
- **Automatic Retry Logic**: Exponential backoff for transient failures
- **Rate Limiting Handling**: Automatic detection and handling of Twilio rate limits
- **Phone Number Validation**: Built-in validation using the `phonenumber` crate
- **Security**: Phone number masking in logs to protect user privacy
- **Error Recovery**: Automatic fallback to mock service on initialization failure

### Technical Features
- Async/await support with Tokio
- Comprehensive error handling with `thiserror`
- Extensive logging with `tracing`
- Clean architecture with trait adapters
- Feature-gated compilation for optional inclusion

## Architecture

### Module Structure
```
server/infra/src/sms/
├── mod.rs                    # Module exports and service factory
├── sms_service.rs           # SmsService trait definition
├── mock_sms.rs              # Mock implementation for development
├── twilio.rs                # Twilio service implementation
├── twilio_trait_adapter.rs  # Adapter for core trait
└── tests/
    └── twilio_tests.rs      # Unit tests
```

### Key Components

1. **TwilioConfig**: Configuration structure for Twilio credentials
2. **TwilioSmsService**: Main service implementation
3. **TwilioSmsServiceAdapter**: Adapter implementing core `SmsServiceTrait`
4. **Retry Logic**: Automatic retry with exponential backoff
5. **Phone Validation**: E.164 format validation and normalization

## Configuration

### Environment Variables

Required environment variables for Twilio:

```bash
# SMS Provider Selection
SMS_PROVIDER=twilio

# Twilio Credentials (required)
TWILIO_ACCOUNT_SID=ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
TWILIO_AUTH_TOKEN=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
TWILIO_FROM_NUMBER=+1234567890  # Must be E.164 format

# Optional Configuration
TWILIO_MAX_RETRIES=3            # Default: 3
TWILIO_RETRY_DELAY_MS=1000      # Default: 1000ms
TWILIO_REQUEST_TIMEOUT_SECS=30  # Default: 30s
```

### Alternative Configuration

The service also supports generic SMS configuration variables for backward compatibility:

```bash
SMS_API_KEY=<your-twilio-account-sid>
SMS_API_SECRET=<your-twilio-auth-token>
SMS_SENDER_ID=<your-twilio-phone-number>
```

## Usage

### Basic Usage

```rust
use re_infra::sms::{TwilioConfig, TwilioSmsService};
use re_infra::sms::sms_service::SmsService;

// Create from environment
let service = TwilioSmsService::from_env()?;

// Send verification code
let message_id = service.send_verification_code(
    "+14155552671",
    "123456"
).await?;
```

### With Dependency Injection

```rust
// In your application state setup
let sms_config = SmsConfig::from_env();
let sms_service: Box<dyn SmsService> = create_sms_service(&sms_config);

// The factory automatically selects the right implementation
// based on SMS_PROVIDER environment variable
```

### Using the Trait Adapter

```rust
use re_infra::sms::TwilioSmsServiceAdapter;
use re_core::services::verification::traits::SmsServiceTrait;

// Create adapter that implements the core trait
let adapter = TwilioSmsServiceAdapter::from_env()?;

// Use with VerificationService
let verification_service = VerificationService::new(
    Box::new(adapter),
    cache_service
);
```

## Error Handling

The service provides comprehensive error handling:

### Retryable Errors
- **429 Rate Limit**: Automatic backoff and retry
- **5xx Server Errors**: Automatic retry with exponential backoff
- **Network Timeouts**: Retry with increased delay

### Non-Retryable Errors
- **400 Bad Request**: Invalid phone number or message format
- **401 Unauthorized**: Invalid credentials
- **404 Not Found**: Invalid account or phone number

### Error Response Format
```rust
InfrastructureError::Sms(String) // Contains detailed error message
```

## Phone Number Validation

The service validates phone numbers in E.164 format:

### Valid Formats
- `+14155552671` (US)
- `+919876543210` (India)
- `+442071234567` (UK)

### Auto-correction
- US numbers without country code are automatically prefixed with `+1`
- Example: `4155552671` → `+14155552671`

### Invalid Formats
- Numbers without `+` prefix (except US auto-correction)
- Numbers shorter than 10 digits
- Numbers longer than 15 digits
- Numbers containing non-digit characters

## Security Considerations

### Phone Number Privacy
- All phone numbers are masked in logs (shows only last 4 digits)
- Example: `+14155552671` → `+******2671`

### Credential Security
- Never log authentication tokens
- Use environment variables for sensitive configuration
- Rotate auth tokens regularly
- Use test credentials for development

### Rate Limiting
- Service respects Twilio's rate limits
- Automatic backoff prevents account suspension
- Configure `TWILIO_MAX_RETRIES` based on your needs

## Testing

### Unit Tests
Run unit tests with:
```bash
cargo test --package re_infra --features twilio-sms
```

### Integration Tests
Integration tests require actual Twilio credentials:
```bash
# Set up test credentials
export TWILIO_ACCOUNT_SID=your_test_sid
export TWILIO_AUTH_TOKEN=your_test_token
export TWILIO_FROM_NUMBER=your_test_number

# Run integration tests
cargo test --package re_infra --features twilio-sms -- --ignored
```

### Mock Service
For development without Twilio credentials:
```bash
SMS_PROVIDER=mock cargo run
```

## Monitoring

### Logging
The service provides detailed logging at various levels:

- **INFO**: Successful SMS sends, service initialization
- **WARN**: Rate limiting, fallback to mock service
- **ERROR**: Send failures, configuration errors
- **DEBUG**: Phone validation, retry attempts

### Metrics to Monitor
- SMS send success rate
- Average send latency
- Retry frequency
- Rate limit occurrences
- Error types and frequencies

## Troubleshooting

### Common Issues

1. **"TWILIO_ACCOUNT_SID not set"**
   - Ensure environment variables are loaded
   - Check `.env` file location
   - Verify variable names are correct

2. **"Invalid phone number format"**
   - Ensure phone numbers are in E.164 format
   - Include country code with `+` prefix
   - Remove any spaces or special characters

3. **"Failed to send SMS after 3 attempts"**
   - Check Twilio account status
   - Verify credentials are correct
   - Check Twilio service status
   - Review rate limiting

4. **"Message exceeds maximum length"**
   - SMS messages limited to 1600 characters
   - Consider splitting long messages
   - Use URL shorteners for links

## Future Enhancements

### Planned Features
- [ ] AWS SNS as fallback provider
- [ ] Message delivery status webhooks
- [ ] SMS templates for localization
- [ ] Batch SMS sending
- [ ] Cost tracking and alerts
- [ ] A/B testing for message content
- [ ] Advanced retry strategies
- [ ] Circuit breaker pattern

### Provider Failover
Future implementation will support automatic failover:
```rust
// Planned implementation
if twilio_service.send_sms().await.is_err() {
    aws_sns_service.send_sms().await
}
```

## References

- [Twilio API Documentation](https://www.twilio.com/docs/sms)
- [E.164 Phone Number Format](https://www.twilio.com/docs/glossary/what-e164)
- [Twilio Rust SDK](https://github.com/neil-lobracco/twilio-rs)
- [Rate Limiting Best Practices](https://www.twilio.com/docs/usage/api/rate-limiting)