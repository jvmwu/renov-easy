# Technology Steering Document - RenovEasy

## Architecture Overview
- **Pattern**: Clean Architecture with Domain-Driven Design
- **Core Business Logic**: Rust (shared across all platforms via FFI)
- **Backend Priority**: Rust backend development precedes native app development
- **Native UI**: Platform-specific implementations (future phase)

## Technology Stack

### Backend Layer (Current Priority)
- **Language**: Rust
- **Build System**: Cargo (standard Rust package manager)
- **Async Runtime**: Tokio for async operations
- **Web Framework**: Actix-web or Axum for RESTful APIs
- **Database ORM**: SQLx or Diesel for MySQL
- **Serialization**: Serde for JSON handling
- **Error Handling**: thiserror and anyhow crates
- **HTTP Client**: reqwest for external services
- **Authentication**: jsonwebtoken for JWT handling
- **Validation**: validator crate for input validation

### FFI Layer (For Platform Integration)
- **C ABI**: Expose Rust functions via C-compatible interface
- **iOS Bridge**: Swift-Rust interop via C bindings
- **Android Bridge**: JNI (Java Native Interface)
- **HarmonyOS Bridge**: NAPI bindings
- **Memory Safety**: Careful handling at FFI boundaries

### Platform-Specific Layers (Future Phase)

#### iOS
- **Language**: Swift
- **UI Framework**: SwiftUI with UIKit fallback
- **FFI Bridge**: Swift-Rust interop via C bindings
- **Maps**: MapKit
- **Networking**: URLSession for platform-specific needs

#### Android
- **Language**: Kotlin
- **UI Framework**: Jetpack Compose
- **FFI Bridge**: JNI (Java Native Interface)
- **Maps**: Google Maps SDK
- **Architecture**: MVVM with ViewModels

#### HarmonyOS
- **Language**: ArkTS/JavaScript
- **UI Framework**: ArkUI
- **FFI Bridge**: NAPI bindings
- **Maps**: Petal Maps or compatible solution

### Infrastructure Services
- **Database**: MySQL for primary data persistence
- **SMS Provider**: Twilio or AWS SNS for phone verification
- **Push Notifications**: FCM/APNS/HMS Push (platform-specific)
- **Real-time**: WebSocket for chat functionality
- **Maps API**: Google Maps API for geocoding and places

### Prototype Reference
- **Purpose**: UI/UX reference for native development
- **Stack**: HTML5 + Tailwind CSS 4.1.11
- **Maps**: Google Maps JavaScript API
- **Icons**: FontAwesome 6.4.0
- **Target**: iPhone 16 Pro (393×852 points)

## Performance Requirements
- **API Response Time**: < 500ms for standard requests
- **Database Queries**: < 100ms for simple queries
- **Concurrent Connections**: Support 1,000+ concurrent WebSocket connections
- **App Launch**: < 2 seconds cold start (future native apps)
- **Backend Scalability**: Horizontal scaling capability

## Development Standards

### Rust Development
- Follow official Rust API Guidelines
- Use rustfmt for consistent formatting
- Apply clippy lints for code quality
- Implement proper error handling with Result types
- Write comprehensive unit tests
- Document public APIs with rustdoc
- Use workspace for multi-crate organization

### Cross-Platform FFI Design
- Unified C-compatible interface layer
- Platform-specific adapters for each OS
- Consistent error codes across platforms
- Memory safety at FFI boundaries
- Proper resource cleanup mechanisms

### Async Programming
- Tokio for async runtime
- Structured concurrency patterns
- Proper cancellation handling
- Backpressure management
- Connection pooling for database

### Database Design
- Normalized schema design (3NF minimum)
- Proper indexing for query performance
- Migration-based schema evolution
- Connection pooling for efficiency
- Prepared statements for security

### API Design
- RESTful principles with clear resource modeling
- Consistent JSON response format
- Versioned API endpoints
- Comprehensive error responses
- OpenAPI/Swagger documentation

### Security Requirements
- **Authentication**: JWT tokens with refresh mechanism
- **Password Storage**: Argon2id hashing (if passwords added later)
- **Data Encryption**: TLS 1.3 for all API communications
- **Input Validation**: Strict validation on all inputs
- **SQL Injection**: Parameterized queries only
- **Rate Limiting**: Per-endpoint rate limits
- **CORS**: Properly configured for web access

### Error Handling
- Unified error types with thiserror
- Consistent error codes for FFI
- Graceful degradation strategies
- User-friendly error messages
- Comprehensive logging


## Third-Party Integrations

### Essential (Phase 1-2)
- **SMS Provider**: Twilio or regional provider for verification
- **Google Maps API**: Geocoding and place search

### Future Integrations
- **Push Notifications**: FCM/APNS/HMS Push
- **Analytics**: Optional tracking solution
- **Crash Reporting**: Sentry or similar
- **Payment Gateway**: Stripe or regional provider (deferred)

## Development Tools
- **Version Control**: Git with feature branch workflow
- **Rust Toolchain**: Latest stable Rust
- **IDE**: VS Code with rust-analyzer
- **Database Tools**: MySQL Workbench or DBeaver
- **API Testing**: Postman or Insomnia
- **Load Testing**: k6 or Apache JMeter

## Testing Strategy

### Backend Testing
- **Unit Tests**: Business logic with 80% coverage target
- **Integration Tests**: API endpoints and database operations
- **Load Tests**: Performance under concurrent load
- **Security Tests**: OWASP API Security Top 10

### Future Native App Testing
- **Unit Tests**: Core functionality
- **UI Tests**: Platform-specific testing
- **E2E Tests**: Critical user journeys

## Deployment Architecture
- **Environment**: Development → Staging → Production
- **Backend Hosting**: Cloud VPS or containerized deployment
- **Database**: Managed MySQL service
- **CI/CD**: GitHub Actions for automated deployment
- **Monitoring**: Application and infrastructure monitoring
- **Logging**: Structured logging with log aggregation

## Technical Constraints
- **Solo Development**: Architecture must be maintainable by one developer
- **Incremental Delivery**: Backend can function independently
- **Platform Independence**: Core logic must work across all platforms
- **Resource Efficiency**: Optimize for cost-effective hosting

## Development Phases

### Phase 1: Backend Foundation (Current)
- Rust project setup with Cargo
- Database schema design and migrations
- Core domain models and business logic
- RESTful API implementation
- Authentication and authorization
- SMS integration for verification

### Phase 2: Backend Completion
- WebSocket server for real-time features
- Google Maps integration
- Advanced search and filtering
- Performance optimization
- API documentation

### Phase 3: Native Development (Future)
- FFI layer implementation
- Platform-specific UI development
- Native feature integration
- App store deployment