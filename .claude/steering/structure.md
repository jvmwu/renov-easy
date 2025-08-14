# Project Structure Steering Document - RenovEasy

## Directory Organization

```
renov-easy/
├── .claude/                     # Claude AI configuration
│   └── steering/                # Project steering documents
├── docs/                        # Documentation (date-based per CLAUDE.md)
│   └── {YYYY_MM_DD}/           # Date-organized folders
│       ├── tasks/              # Task lists and TODOs
│       ├── specs/              # Requirements specifications
│       └── design/             # Design documents
├── platform/                   # Native platform implementations (future)
│   ├── android/                # Android app (Kotlin)
│   ├── ios/                    # iOS app (Swift)
│   └── harmony/                # HarmonyOS app (ArkTS)
├── prototype/                   # HTML/CSS UI reference
│   ├── auth/                   # Authentication flow mockups
│   ├── styles/                 # Design system reference
│   └── *.html                  # Page mockups for UI reference
├── server/                      # Rust backend (current priority)
│   ├── Cargo.toml              # Workspace configuration
│   ├── api/                    # REST API server
│   │   ├── Cargo.toml         # API crate configuration
│   │   ├── src/
│   │   │   ├── main.rs        # Server entry point
│   │   │   ├── routes/        # API route handlers
│   │   │   ├── middleware/    # Request middleware
│   │   │   └── handlers/      # Request handlers
│   │   └── tests/             # API integration tests
│   ├── core/                   # Core business logic
│   │   ├── Cargo.toml         # Core crate configuration
│   │   ├── src/
│   │   │   ├── lib.rs         # Library entry point
│   │   │   ├── domain/        # Domain models and entities
│   │   │   ├── services/      # Business services
│   │   │   ├── repositories/  # Repository traits
│   │   │   └── errors/        # Domain errors
│   │   └── tests/             # Unit tests
│   ├── infrastructure/         # Infrastructure implementations
│   │   ├── Cargo.toml         # Infrastructure crate
│   │   ├── src/
│   │   │   ├── database/      # Database implementations
│   │   │   ├── sms/           # SMS service adapters
│   │   │   ├── maps/          # Maps service adapters
│   │   │   └── cache/         # Caching implementations
│   │   └── tests/
│   ├── ffi/                    # Foreign Function Interface (future)
│   │   ├── Cargo.toml         # FFI crate configuration
│   │   └── src/
│   │       ├── lib.rs         # C-compatible exports
│   │       ├── android.rs     # Android-specific bindings
│   │       ├── ios.rs         # iOS-specific bindings
│   │       └── harmony.rs     # HarmonyOS-specific bindings
│   ├── migrations/             # Database migrations
│   │   └── *.sql              # SQL migration files
│   └── tests/                  # End-to-end tests
│       └── integration/        # Integration test suites
└── tests/                       # Cross-platform E2E tests (future)
    ├── api/                    # API test scenarios
    └── load/                   # Load testing scripts
```

## Cargo Workspace Structure

### Root Cargo.toml
```toml
[workspace]
members = [
    "server/api",
    "server/core",
    "server/infrastructure",
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

### Server Crates

#### Core Domain (`server/core/`)
```
src/
├── domain/
│   ├── entities/               # Business entities
│   │   ├── user.rs
│   │   ├── worker.rs
│   │   ├── order.rs
│   │   └── mod.rs
│   ├── value_objects/         # Value objects
│   │   ├── phone_number.rs
│   │   ├── location.rs
│   │   └── mod.rs
│   └── events/                # Domain events
├── services/
│   ├── auth_service.rs        # Authentication logic
│   ├── order_service.rs       # Order management
│   ├── user_service.rs        # User management
│   └── worker_service.rs      # Worker management
├── repositories/               # Repository traits
│   ├── user_repository.rs
│   └── order_repository.rs
└── errors/                    # Domain errors
    └── domain_error.rs
```

#### API Server (`server/api/`)
```
src/
├── routes/
│   ├── auth.rs               # Authentication routes
│   ├── users.rs              # User management routes
│   ├── workers.rs            # Worker routes
│   ├── orders.rs             # Order routes
│   └── mod.rs
├── handlers/
│   └── error_handler.rs      # Global error handling
├── middleware/
│   ├── auth.rs               # JWT authentication
│   ├── cors.rs               # CORS configuration
│   └── rate_limit.rs         # Rate limiting
└── dto/                       # Data transfer objects
    ├── request/
    └── response/
```

#### Infrastructure (`server/infrastructure/`)
```
src/
├── database/
│   ├── mysql/
│   │   ├── connection.rs     # Connection pool
│   │   └── repositories/     # Repository implementations
│   └── migrations.rs         # Migration runner
├── sms/
│   ├── twilio.rs             # Twilio adapter
│   └── mock.rs               # Mock for testing
└── maps/
    └── google_maps.rs        # Google Maps adapter
```

## API Design Standards

### RESTful Endpoints
```
GET    /api/v1/users           # List users
POST   /api/v1/users           # Create user
GET    /api/v1/users/{id}      # Get user
PUT    /api/v1/users/{id}      # Update user
DELETE /api/v1/users/{id}      # Delete user

POST   /api/v1/auth/login      # User login
POST   /api/v1/auth/verify     # SMS verification
POST   /api/v1/auth/refresh    # Token refresh

GET    /api/v1/orders          # List orders
POST   /api/v1/orders          # Create order
GET    /api/v1/orders/{id}     # Get order details
PUT    /api/v1/orders/{id}     # Update order status

WS     /api/v1/ws/chat         # WebSocket for chat
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