# Project Structure Steering Document - RenovEasy

## Directory Organization

```
renov-easy/
├── .claude/                       # Claude AI configuration
│   ├── steering/                  # Project steering documents
│   └── specs/                     # Feature specifications
├── docs/                          # Documentation (date-based)
│   └── {YYYY_MM_DD}/              # Date-organized folders
├── platform/                      # Native platform implementations (future)
│   ├── android/                   # Android app (Kotlin)
│   ├── ios/                       # iOS app (Swift) 
│   └── harmony/                   # HarmonyOS app (ArkTS)
├── prototype/                     # HTML/CSS UI reference
├── server/                        # Rust backend
│   ├── Cargo.toml                 # Workspace configuration
│   ├── api/                       # REST API server ✅
│   ├── core/                      # Business logic ✅
│   ├── infra/                     # Infrastructure ✅
│   ├── shared/                    # Shared utilities ✅
│   ├── ffi/                       # FFI bindings 📝
│   └── migrations/                # Database migrations ✅
└── tests/                         # E2E tests (future)
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

### Server Crates Architecture

#### Core Module (`server/core/`) ✅
**Purpose**: Domain logic, business rules, and service interfaces
- **Domain Layer**: Entities, value objects, domain events
- **Service Layer**: Authentication, verification, audit services  
- **Repository Interfaces**: Data access contracts
- **Error Handling**: Domain-specific error types

#### API Module (`server/api/`) ✅  
**Purpose**: HTTP server, routing, and request handling
- **Routes**: RESTful endpoint definitions
- **Middleware**: Security, auth, rate limiting, CORS
- **DTOs**: Request/response data structures
- **I18n**: Multi-language support

#### Infrastructure Module (`server/infra/`) ✅
**Purpose**: External service integrations and data persistence
- **Database**: MySQL repository implementations
- **Cache**: Redis for sessions and temporary data
- **SMS**: Third-party SMS service integration
- **File Storage**: S3-compatible object storage (future)

#### Shared Module (`server/shared/`) ✅
**Purpose**: Cross-cutting concerns and utilities
- **Configuration**: Unified config structures
- **Common Types**: Shared data types and responses
- **Utilities**: Validation, formatting, helpers
- **Error Types**: Common error definitions

#### FFI Module (`server/ffi/`) 📝
**Purpose**: Platform bindings for mobile apps
- **Android**: JNI bindings for Kotlin/Java
- **iOS**: C-compatible bindings for Swift
- **HarmonyOS**: NAPI bindings for ArkTS

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

### Environment Strategy
- **Development**: Local environment with mock services
- **Staging**: Production-like with test data
- **Production**: Live environment with real services

### Configuration Sources
1. Environment variables (highest priority)
2. Configuration files (`.toml`, `.yaml`)
3. Default values (fallback)

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
