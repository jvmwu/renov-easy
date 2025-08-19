# Project Structure Steering Document - RenovEasy

## Directory Organization

```
renov-easy/
├── .claude/                       # Claude AI configuration
│   └── steering/                  # Project steering documents
├── docs/                          # Documentation (date-based per CLAUDE.md)
│   └── {YYYY_MM_DD}/              # Date-organized folders
│       ├── tasks/                 # Task lists and TODOs
│       ├── specs/                 # Requirements specifications
│       └── design/                # Design documents
├── platform/                      # Native platform implementations (future)
│   ├── android/                   # Android app (Kotlin)
│   ├── ios/                       # iOS app (Swift)
│   └── harmony/                   # HarmonyOS app (ArkTS)
├── prototype/                     # HTML/CSS UI reference
│   ├── auth/                      # Authentication flow mockups
│   ├── styles/                    # Design system reference
│   └── *.html                     # Page mockups for UI reference
├── server/                        # Rust backend
│   ├── Cargo.toml                 # Workspace configuration
│   ├── api/                       # REST API server ✅
│   │   ├── Cargo.toml             # API crate configuration
│   │   ├── i18n/                  # Internationalization ✅
│   │   │   └── error_messages.toml # Error message translations
│   │   ├── src/
│   │   │   ├── main.rs            # Server entry point
│   │   │   ├── lib.rs             # Library exports
│   │   │   ├── app.rs             # Application setup
│   │   │   ├── config.rs          # Configuration management
│   │   │   ├── dto/               # Data transfer objects ✅
│   │   │   │   ├── auth.rs        # Authentication DTOs
│   │   │   │   └── error.rs       # Error response DTOs
│   │   │   ├── handlers/          # Request handlers ✅
│   │   │   │   └── error.rs       # Error handling with i18n
│   │   │   ├── i18n/              # I18n module ✅
│   │   │   │   └── mod.rs         # Language support
│   │   │   ├── middleware/        # Security middleware ✅
│   │   │   │   ├── auth.rs        # JWT authentication
│   │   │   │   ├── cors.rs        # CORS configuration
│   │   │   │   ├── rate_limit.rs  # Rate limiting
│   │   │   │   └── security.rs    # Security headers
│   │   │   └── routes/            # API routes ✅
│   │   │       └── auth/          # Authentication endpoints
│   │   │           ├── send_code.rs    # SMS verification
│   │   │           ├── verify_code.rs  # Code validation
│   │   │           ├── select_type.rs  # User type selection
│   │   │           ├── refresh.rs      # Token refresh
│   │   │           └── logout.rs       # User logout
│   │   └── tests/                 # Comprehensive test suite ✅
│   │       ├── i18n_test.rs      # I18n tests
│   │       ├── error_handling_test.rs  # Error handling tests
│   │       └── auth_middleware_test.rs # Auth tests
│   ├── core/                      # Core business logic ✅
│   │   ├── Cargo.toml             # Core crate configuration
│   │   ├── src/
│   │   │   ├── lib.rs             # Library entry point
│   │   │   ├── domain/            # Domain models ✅
│   │   │   │   ├── entities/      # Business entities
│   │   │   │   │   ├── user.rs    # User entity
│   │   │   │   │   ├── token.rs   # Token entity
│   │   │   │   │   ├── audit.rs   # Audit log entity ✅
│   │   │   │   │   ├── verification_code.rs # Verification codes
│   │   │   │   │   └── tests/     # Entity unit tests
│   │   │   │   ├── events/        # Domain events ✅
│   │   │   │   └── value_objects/ # Value objects ✅
│   │   │   │       └── tests/     # Value object tests
│   │   │   ├── services/          # Business services ✅
│   │   │   │   ├── auth/          # Authentication service
│   │   │   │   ├── audit/         # Audit service ✅
│   │   │   │   ├── token/         # Token management
│   │   │   │   ├── verification/  # Verification service
│   │   │   │   └── tests/         # Service unit tests
│   │   │   ├── repositories/      # Repository traits ✅
│   │   │   │   ├── user/          # User repository trait
│   │   │   │   ├── token/         # Token repository trait
│   │   │   │   ├── audit/         # Audit repository trait ✅
│   │   │   │   └── tests/         # Repository tests
│   │   │   └── errors/            # Domain errors ✅
│   │   │       ├── mod.rs         # Error module
│   │   │       └── types.rs       # Error type definitions
│   │   └── tests/                 # Integration tests
│   ├── infra/                     # Infrastructure layer ✅
│   │   ├── Cargo.toml             # Infrastructure crate
│   │   ├── README.md              # Infrastructure documentation
│   │   ├── src/
│   │   │   ├── lib.rs             # Library exports
│   │   │   ├── cache/             # Redis cache ✅
│   │   │   │   ├── redis_client.rs # Redis client
│   │   │   │   ├── verification_cache.rs # Verification cache
│   │   │   │   └── tests/         # Cache tests
│   │   │   ├── database/          # Database layer ✅
│   │   │   │   ├── connection.rs  # Connection pool
│   │   │   │   ├── mysql/         # MySQL implementations
│   │   │   │   │   ├── user_repository_impl.rs
│   │   │   │   │   └── token_repository_impl.rs
│   │   │   │   └── tests/         # Database tests
│   │   │   └── sms/               # SMS service ✅
│   │   │       ├── sms_service.rs # SMS service trait
│   │   │       ├── mock_sms.rs    # Mock implementation
│   │   │       └── tests/         # SMS tests
│   │   ├── examples/              # Usage examples
│   │   └── tests/                 # Integration tests ✅
│   │       ├── database_integration.rs
│   │       ├── redis_integration.rs
│   │       └── sms_integration.rs
│   ├── shared/                    # Shared utilities ✅
│   │   ├── Cargo.toml             # Shared crate configuration
│   │   ├── src/
│   │   │   ├── lib.rs             # Library exports
│   │   │   ├── config/            # Configuration types
│   │   │   │   ├── mod.rs         # Config module exports
│   │   │   │   ├── auth.rs        # Authentication config (JWT, OAuth2, Session)
│   │   │   │   ├── cache.rs       # Cache config (Redis, Memory)
│   │   │   │   ├── database.rs    # Database config
│   │   │   │   ├── environment.rs # Environment config (Dev/Staging/Prod)
│   │   │   │   ├── rate_limit.rs  # Rate limiting config
│   │   │   │   └── server.rs      # Server config (host, port, TLS)
│   │   │   ├── errors/            # Common errors
│   │   │   │   └── mod.rs         # Error exports
│   │   │   ├── types/             # Common types
│   │   │   │   ├── mod.rs         # Type exports
│   │   │   │   ├── common.rs      # Common type definitions
│   │   │   │   ├── language.rs    # Language support types
│   │   │   │   ├── pagination.rs  # Pagination types
│   │   │   │   └── response.rs    # API response types (ErrorResponse, etc.)
│   │   │   └── utils/             # Utility functions
│   │   │       ├── mod.rs         # Utils exports
│   │   │       ├── phone.rs       # Phone number utilities
│   │   │       └── validation.rs  # Input validation utilities
│   ├── ffi/                       # Foreign Function Interface ✅
│   │   ├── android/               # Android bindings
│   │   ├── ios/                   # iOS bindings
│   │   └── harmony/               # HarmonyOS bindings
│   ├── migrations/                # Database migrations ✅
│   │   ├── 001_create_users_table.sql
│   │   └── 002_create_tokens_audit_tables.sql
│   └── build/                     # Build artifacts
└── tests/                         # Cross-platform E2E tests (future)
    ├── api/                       # API test scenarios
    └── load/                      # Load testing scripts
```

## Cargo Workspace Structure

### Root Cargo.toml
```toml
[workspace]
members = [
    "server/api",
    "server/core",
    "server/infra",
    "server/shared",
    "server/ffi"
]

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["RenovEasy Team"]

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
sqlx = { version = "0.7", features = ["mysql", "runtime-tokio"] }
```

## Naming Conventions

### Rust/Backend Files
- **Modules**: snake_case (e.g., `user_service.rs`)
- **Structs/Enums**: PascalCase (e.g., `UserProfile`)
- **Functions**: snake_case (e.g., `create_user`)
- **Constants**: SCREAMING_SNAKE_CASE (e.g., `MAX_RETRIES`)
- **Database Tables**: snake_case (e.g., `user_profiles`)
- **API Endpoints**: kebab-case (e.g., `/api/v1/user-profiles`)

### Documentation
- **Markdown Files**: kebab-case (e.g., `api-design.md`)
- **Date Folders**: YYYY_MM_DD format (e.g., `2025_08_14`)

### Future Platform Code
- **Swift**: PascalCase for types, camelCase for methods
- **Kotlin**: PascalCase for classes, camelCase for functions
- **ArkTS**: PascalCase for components, camelCase for functions

## Module Organization

### Server Crates (Current Implementation Status)

#### Core Domain (`server/core/`)
```
src/
├── domain/
│   ├── entities/                  # Business entities ✅
│   │   ├── user.rs                # User entity with roles
│   │   ├── token.rs               # JWT token management
│   │   ├── audit.rs               # Audit logging entity
│   │   ├── verification_code.rs   # SMS verification
│   │   └── tests/                 # Entity unit tests
│   ├── value_objects/             # Value objects ✅
│   │   ├── phone_number.rs        # Phone validation
│   │   ├── email.rs               # Email validation
│   │   ├── user_role.rs           # User roles enum
│   │   └── tests/                 # Value object tests
│   └── events/                    # Domain events ✅
│       └── audit_event.rs         # Audit event types
├── services/                      # Business services ✅
│   ├── auth/                      # Authentication service
│   │   ├── service.rs             # Auth business logic
│   │   ├── rate_limiter.rs        # Rate limiting trait
│   │   └── tests/
│   ├── audit/                     # Audit service ✅
│   │   ├── service.rs             # Audit logging
│   │   └── tests/
│   ├── token/                     # Token management
│   │   ├── service.rs             # JWT operations
│   │   └── tests/
│   └── verification/              # Verification service
│       ├── service.rs             # SMS code verification
│       └── tests/
├── repositories/                  # Repository traits ✅
│   ├── user/                      # User repository
│   ├── token/                     # Token repository
│   └── audit/                     # Audit repository
└── errors/                        # Domain errors ✅
    ├── mod.rs                     # Error aggregation
    └── types.rs                   # Error type definitions
```

#### API Server (`server/api/`)
```
src/
├── routes/
│   └── auth/                      # Authentication routes ✅
│       ├── send_code.rs           # POST /api/v1/auth/send-code
│       ├── verify_code.rs         # POST /api/v1/auth/verify-code
│       ├── select_type.rs         # POST /api/v1/auth/select-type
│       ├── refresh.rs             # POST /api/v1/auth/refresh
│       └── logout.rs              # POST /api/v1/auth/logout
├── handlers/
│   └── error.rs                   # Global error handling with i18n ✅
├── middleware/                    # Security middleware ✅
│   ├── auth.rs                    # JWT authentication
│   ├── cors.rs                    # CORS configuration
│   ├── rate_limit.rs              # Rate limiting (Redis-based)
│   └── security.rs                # Security headers
├── dto/                           # Data transfer objects ✅
│   ├── auth.rs                    # Auth request/response DTOs
│   └── error.rs                   # Error response DTOs
├── i18n/                          # Internationalization ✅
│   └── mod.rs                     # Language support (EN/ZH)
└── config.rs                      # Application configuration
```

#### Infrastructure (`server/infra/`)
```
src/
├── cache/                         # Redis implementation ✅
│   ├── redis_client.rs            # Redis connection pool
│   ├── verification_cache.rs      # Verification code cache
│   └── tests/
├── database/                      # MySQL implementation ✅
│   ├── connection.rs              # Connection pool (sqlx)
│   ├── mysql/
│   │   ├── user_repository_impl.rs  # User CRUD
│   │   └── token_repository_impl.rs # Token storage
│   └── tests/
└── sms/                           # SMS service ✅
    ├── sms_service.rs             # SMS service trait
    ├── mock_sms.rs                # Mock for development
    └── tests/
```

#### Shared Module (`server/shared/`)
```
src/
├── lib.rs                         # Library exports
├── config/                        # Shared configuration types ✅
│   ├── mod.rs                     # Module exports
│   ├── auth.rs                    # Authentication config
│   │   ├── JwtConfig              # JWT configuration
│   │   ├── OAuth2Config           # OAuth2 provider config
│   │   ├── SessionConfig          # Session management
│   │   └── AuthConfig             # Complete auth config
│   ├── cache.rs                   # Cache configuration
│   │   ├── CacheConfig            # Redis cache config
│   │   ├── MemoryCacheConfig      # In-memory cache config
│   │   └── CacheStrategyConfig    # Cache strategy selection
│   ├── database.rs                # Database configuration
│   │   └── DatabaseConfig         # Connection pool config
│   ├── environment.rs             # Environment configuration
│   │   └── Environment            # Dev/Staging/Production
│   ├── rate_limit.rs              # Rate limiting configuration
│   │   ├── RateLimitConfig        # Main rate limit config
│   │   ├── SmsRateLimits          # SMS-specific limits
│   │   ├── ApiRateLimits          # API rate limits
│   │   └── AuthRateLimits         # Auth rate limits
│   └── server.rs                  # Server configuration
│       ├── ServerConfig           # HTTP server config
│       └── TlsConfig              # TLS/SSL config
├── errors/                        # Common error types ✅
│   └── mod.rs                     # Error definitions
├── types/                         # Common type definitions ✅
│   ├── mod.rs                     # Type exports
│   ├── common.rs                  # Common types
│   │   ├── Id                     # UUID wrapper
│   │   ├── Timestamp              # DateTime wrapper
│   │   └── PhoneNumber            # Phone number type
│   ├── language.rs                # Language support
│   │   ├── Language               # Language enum
│   │   └── LocalizedString        # Multi-language strings
│   ├── pagination.rs              # Pagination support
│   │   ├── PaginationRequest      # Page request params
│   │   └── PaginatedResponse      # Paginated response
│   └── response.rs                # API response types
│       ├── ApiResponse            # Standard API response
│       ├── ErrorResponse          # Error response structure
│       ├── DetailedResponse       # Detailed response with meta
│       ├── BatchResponse          # Batch operation response
│       └── HealthResponse         # Health check response
└── utils/                         # Utility functions ✅
    ├── mod.rs                     # Utils exports
    ├── phone.rs                   # Phone validation/formatting
    │   ├── validate_phone()       # Phone number validation
    │   ├── format_phone()         # Phone formatting
    │   └── hash_phone()           # Phone hashing for privacy
    └── validation.rs              # Input validation
        ├── validate_email()       # Email validation
        ├── validate_password()    # Password strength check
        └── sanitize_input()       # Input sanitization
```

#### FFI Module (`server/ffi/`)
```
├── android/                       # Android JNI bindings
├── ios/                           # iOS bindings
└── harmony/                       # HarmonyOS bindings
```

## API Design Standards

### RESTful Endpoints
```
GET    /api/v1/users               # List users
POST   /api/v1/users               # Create user
GET    /api/v1/users/{id}          # Get user
PUT    /api/v1/users/{id}          # Update user
DELETE /api/v1/users/{id}          # Delete user

POST   /api/v1/auth/login          # User login
POST   /api/v1/auth/verify         # SMS verification
POST   /api/v1/auth/refresh        # Token refresh

GET    /api/v1/orders              # List orders
POST   /api/v1/orders              # Create order
GET    /api/v1/orders/{id}         # Get order details
PUT    /api/v1/orders/{id}         # Update order status

WS     /api/v1/ws/chat             # WebSocket for chat
```

### Response Format
```json
{
  "success": true,
  "data": {},
  "error": null,
  "timestamp": "2025-08-14T10:00:00Z"
}
```

## Testing Structure

### Unit Tests
- Located alongside source files
- Named `{module}_test.rs` or in `tests` module
- Focus on business logic isolation

### Integration Tests
- Located in `tests/` directory of each crate
- Test database operations and external services
- Use test database and mock services

### E2E Tests
- Located in root `/tests/` directory
- Test complete user workflows
- Run against staging environment

## Git Conventions

### Branch Strategy
- `main`: Production-ready code
- `develop`: Integration branch
- `feature/backend-{feature}`: Backend features
- `feature/ios-{feature}`: iOS features (future)
- `fix/{issue-number}`: Bug fixes

### Commit Format
```
<type>(<scope>): <subject>

Types: feat, fix, docs, style, refactor, test, chore
Scope: backend, ios, android, harmony, api, db

Example: feat(backend): implement user authentication
```

## Configuration Management

### Environment Files
```
.env.development     # Local development
.env.staging        # Staging environment
.env.production     # Production (never commit)
```

### Configuration Structure
```rust
// server/api/src/config.rs
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub sms_api_key: String,
    pub google_maps_key: String,
    pub server_port: u16,
}
```

## Documentation Requirements

### Code Documentation
- All public APIs must have rustdoc comments
- Complex algorithms need inline explanations
- Configuration examples in comments

### API Documentation
- OpenAPI/Swagger specification
- Postman collection for testing
- README with setup instructions

## Security Practices

### Code Security
- No hardcoded secrets or credentials
- Input validation on all endpoints
- Parameterized database queries only
- Rate limiting on sensitive endpoints

### Dependency Management
- Regular `cargo audit` for vulnerabilities
- Conservative dependency updates
- Minimal external dependencies

## Performance Guidelines

### Backend Optimization
- Database query optimization with indexes
- Connection pooling for all external services
- Async/await for I/O operations
- Response caching where appropriate

### Monitoring
- Request/response logging
- Performance metrics collection
- Error tracking and alerting
- Database query performance monitoring
