# Infrastructure Module

## Overview

The infrastructure module provides concrete implementations of external services and data persistence for the RenovEasy backend. It follows the principles of Clean Architecture, keeping infrastructure concerns separate from business logic.

## Module Structure

```
infrastructure/
├── src/
│   ├── cache/              # Redis cache implementations
│   │   ├── redis_client.rs      # Redis connection pool and basic operations
│   │   └── verification_cache.rs # SMS verification code caching
│   ├── database/           # Database implementations
│   │   ├── connection.rs        # MySQL connection pool
│   │   └── mysql/               # MySQL repository implementations
│   │       ├── user_repository_impl.rs
│   │       └── token_repository_impl.rs
│   ├── sms/                # SMS service implementations
│   │   ├── sms_service.rs      # SMS service trait
│   │   └── mock_sms.rs         # Mock SMS for development
│   ├── config.rs           # Configuration structures
│   └── errors.rs           # Infrastructure error types
```

## Components

### Cache Services

#### Redis Client (`cache/redis_client.rs`)
Provides connection pooling and basic Redis operations with automatic retry logic.

**Features:**
- Connection pooling with configurable size
- Exponential backoff retry mechanism
- Basic operations: set_with_expiry, get, delete, exists, ttl, increment
- Health check functionality

**Configuration:**
```rust
let config = CacheConfig {
    url: "redis://localhost:6379".to_string(),
    pool_size: 5,
    default_ttl: 3600,
};
let client = RedisClient::new(config).await?;
```

#### Verification Cache (`cache/verification_cache.rs`)
Secure storage and validation of SMS verification codes.

**Features:**
- 5-minute code expiration
- Maximum 3 verification attempts tracking
- SHA-256 hashing for secure code storage
- Automatic cleanup after successful verification
- Phone number masking in logs

**Usage:**
```rust
let cache = VerificationCache::new(redis_client);

// Store verification code
cache.store_code("+1234567890", "123456").await?;

// Verify code (returns true/false)
let is_valid = cache.verify_code("+1234567890", user_input).await?;

// Check remaining attempts
let attempts = cache.get_remaining_attempts("+1234567890").await?;
```

### Database Services

#### Connection Pool (`database/connection.rs`)
MySQL connection pool management using SQLx.

**Features:**
- Connection pooling with configurable limits
- Automatic reconnection
- Health check endpoint
- Query timeout configuration

#### Repository Implementations
- **UserRepository** (`database/mysql/user_repository_impl.rs`): User CRUD operations with phone hash security
- **TokenRepository** (`database/mysql/token_repository_impl.rs`): Refresh token management

### SMS Services

#### SMS Service Trait (`sms/sms_service.rs`)
Defines the interface for SMS providers.

**Features:**
- Phone number validation (E.164 format)
- Phone number masking for logs
- Async trait definition
- Provider-agnostic interface

#### Mock SMS Service (`sms/mock_sms.rs`)
Development environment SMS implementation.

**Features:**
- Console output for verification codes
- Message counter
- Failure simulation for testing
- Formatted display output

**Usage:**
```rust
// Create service based on configuration
let sms_service = create_sms_service(&sms_config);

// Send SMS
sms_service.send_sms("+1234567890", "Your code is: 123456").await?;
```

## Configuration

All infrastructure services are configured through the `Config` structure:

```rust
pub struct Config {
    pub database: DatabaseConfig,
    pub cache: CacheConfig,
    pub sms: SmsConfig,
}

pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: Duration,
}

pub struct CacheConfig {
    pub url: String,
    pub pool_size: usize,
    pub default_ttl: u64,
}

pub struct SmsConfig {
    pub provider: SmsProvider,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
}
```

## Error Handling

All infrastructure errors are unified under `InfrastructureError`:

```rust
pub enum InfrastructureError {
    DatabaseError(String),
    CacheError(String),
    SmsError(String),
    ConfigError(String),
    ConnectionError(String),
}
```

## Testing

### Unit Tests
```bash
# Run all infrastructure tests
cargo test --package infrastructure

# Run specific module tests
cargo test --package infrastructure cache::
cargo test --package infrastructure database::
cargo test --package infrastructure sms::
```

### Integration Tests
```bash
# Requires Redis and MySQL running
cargo test --package infrastructure --test '*' -- --ignored
```

### Examples
```bash
# Redis verification example
cargo run --example redis_verification

# SMS service example
cargo run --example sms_demo
```

## Environment Variables

Required environment variables for production:

```env
# Database
DATABASE_URL=mysql://user:password@localhost:3306/renoveasy
DATABASE_MAX_CONNECTIONS=10
DATABASE_MIN_CONNECTIONS=2

# Redis Cache
REDIS_URL=redis://localhost:6379
REDIS_POOL_SIZE=5

# SMS Provider (development uses mock)
SMS_PROVIDER=mock  # Options: mock, twilio, aws_sns
SMS_API_KEY=your_api_key
SMS_API_SECRET=your_api_secret
```

## Security Considerations

1. **Phone Number Privacy**: All phone numbers are hashed using SHA-256 before database storage
2. **Logging**: Phone numbers are masked in logs (only last 4 digits shown)
3. **Verification Codes**: Stored as SHA-256 hashes in Redis
4. **Connection Security**: TLS/SSL support for database and Redis connections
5. **Secrets Management**: All API keys and passwords from environment variables

## Performance Optimization

1. **Connection Pooling**: All external services use connection pools
2. **Async Operations**: Full async/await support with Tokio
3. **Retry Logic**: Exponential backoff for transient failures
4. **Caching Strategy**: 5-minute TTL for verification codes
5. **Query Optimization**: Prepared statements and indexed queries

## Future Enhancements

- [ ] Add Twilio SMS provider implementation
- [ ] Add AWS SNS SMS provider implementation
- [ ] Implement distributed caching with Redis Cluster
- [ ] Add metrics collection (Prometheus)
- [ ] Implement circuit breaker pattern for external services
- [ ] Add database migration tooling
- [ ] Implement event sourcing for audit logs

## Dependencies

Key dependencies used in this module:

- `sqlx` - Async SQL toolkit for MySQL
- `redis` - Redis client with async support
- `async-trait` - Async trait definitions
- `sha2` - SHA-256 hashing
- `tokio` - Async runtime
- `thiserror` - Error handling
- `serde` - Serialization/deserialization
- `chrono` - Date/time handling
- `uuid` - UUID generation