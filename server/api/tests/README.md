# API Integration Tests

This directory contains integration tests for the API module.

## Test Organization

### Authentication Tests
- `send_code_test.rs` - Tests for SMS verification code sending
- `logout_test.rs` - Tests for user logout functionality
- `auth_middleware_test.rs` - Tests for authentication middleware

### Error Handling Tests
- `error_handling_test.rs` - Tests for bilingual error handling system
- `test_bilingual_errors.rs` - Legacy bilingual error tests (to be merged)

### Internationalization Tests
- `i18n_test.rs` - Tests for i18n message system and formatting

### Middleware Tests
- `middleware_integration_test.rs` - Integration tests for all middleware components

## Running Tests

```bash
# Run all tests in the api module
cargo test -p api

# Run specific test file
cargo test -p api --test i18n_test

# Run with output
cargo test -p api -- --nocapture

# Run specific test function
cargo test -p api test_auth_error_invalid_verification_code
```

## Test Coverage Areas

1. **Error Handling**
   - All error types and their HTTP status codes
   - Bilingual message support (English/Chinese)
   - Parameter formatting in error messages

2. **Authentication**
   - SMS code verification flow
   - Token generation and validation
   - User authentication states

3. **Middleware**
   - Rate limiting
   - CORS configuration
   - Security headers
   - Authentication guards

4. **Internationalization**
   - Message loading from configuration
   - Language detection from headers
   - Dynamic message formatting

## Notes

- Integration tests use the actual service implementations
- Mock implementations are used for external services (SMS, Redis)
- Tests should be independent and not rely on external state