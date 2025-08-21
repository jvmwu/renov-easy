# Technology Steering Document - RenovEasy

## Architecture Overview

### System Architecture Diagram
```
┌─────────────────────────────────────────────────────────────────┐
│                     Client Layer (Mobile Apps)                   │
├─────────────────────────────────────────────────────────────────┤
│   iOS App          Android App         HarmonyOS App            │
│   (Swift)          (Kotlin)            (ArkTS)                  │
│   SwiftUI          Jetpack Compose     ArkUI                    │
└──────┬──────────────────┬──────────────────┬───────────────────┘
       │                  │                  │
       └──────────────────┴──────────────────┘
                          │
                    FFI Bridge Layer
┌─────────────────────────────────────────────────────────────────┐
│                   Rust FFI Bindings (C ABI)                     │
│         iOS Bridge │ JNI Bridge │ NAPI Bridge                   │
└─────────────────────────────────────────────────────────────────┘
                          │
                    API Gateway Layer
┌─────────────────────────────────────────────────────────────────┐
│                    REST API Gateway                              │
│                   (Actix-web/Axum)                              │
├─────────────────────────────────────────────────────────────────┤
│  Rate Limiting │ Auth Middleware │ CORS │ Security Headers      │
│  Load Balancer │ API Versioning  │ Request Validation          │
└─────────────────────────────────────────────────────────────────┘
                          │
                 Business Logic Layer
┌─────────────────────────────────────────────────────────────────┐
│                   Core Business Services                         │
├───────────────────┬────────────────┬────────────────────────────┤
│  Auth Service     │  Order Service │  Matching Engine          │
│  • JWT/OAuth2     │  • CRUD Ops    │  • Location-based         │
│  • SMS OTP        │  • Workflow    │  • Score Algorithm        │
│  • Session Mgmt   │  • Status FSM  │  • ML Recommendations     │
├───────────────────┼────────────────┼────────────────────────────┤
│  User Service     │  Chat Service  │  Payment Service          │
│  • Profile Mgmt   │  • WebSocket   │  • Stripe/Alipay          │
│  • Verification   │  • Real-time   │  • Escrow                 │
│  • Ratings        │  • History     │  • Invoicing              │
└───────────────────┴────────────────┴────────────────────────────┘
                          │
                    Domain Layer
┌─────────────────────────────────────────────────────────────────┐
│              Domain Models & Business Rules                      │
├─────────────────────────────────────────────────────────────────┤
│  Entities        Value Objects       Domain Events              │
│  • User          • PhoneNumber       • OrderCreated             │
│  • Order         • Email             • PaymentReceived          │
│  • Worker        • Money             • JobCompleted             │
│  • Rating        • Location          • UserRegistered           │
└─────────────────────────────────────────────────────────────────┘
                          │
                 Infrastructure Layer
┌─────────────────────────────────────────────────────────────────┐
│                    Data Persistence Layer                        │
├────────────────┬────────────────┬────────────────┬──────────────┤
│    MySQL       │     Redis       │  Elasticsearch │   S3/OSS    │
│  • User Data   │  • Sessions     │  • Full-text   │  • Images   │
│  • Orders      │  • OTP Cache    │  • Analytics   │  • Documents│
│  • Transactions│  • Rate Limit   │  • Logs        │  • Backups  │
└────────────────┴────────────────┴────────────────┴──────────────┘
                          │
                  External Services
┌─────────────────────────────────────────────────────────────────┐
│                    Third-Party Integrations                      │
├────────────────┬────────────────┬────────────────┬──────────────┤
│  SMS Gateway   │  Google Maps    │  Push Services │  Payment    │
│  • Twilio      │  • Geocoding    │  • FCM (Android)│ • Stripe   │
│  • AWS SNS     │  • Places API   │  • APNS (iOS)  │  • Alipay  │
│  • Aliyun SMS  │  • Directions   │  • HMS (Huawei)│  • WeChat  │
└────────────────┴────────────────┴────────────────┴──────────────┘
                          │
                Monitoring & Operations
┌─────────────────────────────────────────────────────────────────┐
│                 DevOps & Monitoring Infrastructure               │
├────────────────┬────────────────┬────────────────┬──────────────┤
│  Logging       │  Monitoring     │  Alerting      │  CI/CD      │
│  • ELK Stack   │  • Prometheus   │  • PagerDuty   │  • GitHub   │
│  • Structured  │  • Grafana      │  • Slack       │    Actions  │
│  • Audit Logs  │  • APM          │  • Email       │  • Docker   │
└────────────────┴────────────────┴────────────────┴──────────────┘
```

### Architecture Patterns
- **Pattern**: Clean Architecture with Domain-Driven Design (DDD)
- **Core Business Logic**: Rust (shared across all platforms via FFI)
- **Backend Priority**: Rust backend development precedes native app development
- **Native UI**: Platform-specific implementations (future phase)
- **Communication**: RESTful APIs with WebSocket for real-time features
- **Data Flow**: Unidirectional with event-driven updates

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

### Backend Performance Targets
- **API Response Time**: P50 < 200ms, P95 < 500ms, P99 < 1s
- **Database Performance**: 
  - Simple queries < 50ms
  - Complex joins < 200ms  
  - Batch operations < 500ms
- **Concurrent Users**: 10,000+ simultaneous connections
- **WebSocket**: 5,000+ concurrent chat sessions
- **Geographic Search**: < 100ms for 10km radius queries
- **Throughput**: 1,000+ requests/second per server instance

### Mobile App Performance
- **Cold Start**: < 2 seconds on mid-range devices
- **Screen Transitions**: < 300ms animation time
- **Image Loading**: Progressive loading with placeholders
- **Offline Mode**: Core features work without network
- **Battery Usage**: < 5% drain per hour of active use
- **Memory Footprint**: < 150MB for normal operation

### Scalability Architecture
- **Horizontal Scaling**: Stateless services for easy scaling
- **Database Sharding**: Geographic sharding for user data
- **Cache Strategy**: Multi-layer caching (CDN, Redis, Application)
- **Load Balancing**: Geographic load distribution
- **Auto-scaling**: CPU/Memory based auto-scaling policies

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

### Security Architecture

#### Authentication & Authorization
- **Multi-Factor Auth**: SMS OTP + optional biometric (Face ID/fingerprint)
- **Token Management**: 
  - JWT with RS256 signing algorithm
  - Access tokens: 15-minute expiry with automatic refresh
  - Refresh tokens: 30-day expiry with rotation on use
  - Token blacklisting for immediate revocation
- **Session Security**: Device fingerprinting, IP validation, geolocation checks

#### Data Protection
- **Encryption at Rest**: AES-256-GCM for sensitive data fields
- **Encryption in Transit**: TLS 1.3 minimum, certificate pinning on mobile
- **PII Protection**: Field-level encryption for phone numbers, ID cards
- **Key Management**: HashiCorp Vault or AWS KMS for key rotation
- **Data Masking**: Automatic PII masking in logs and non-production environments

#### Application Security
- **Input Validation**: 
  - Schema validation with JSON Schema
  - SQL injection prevention via parameterized queries
  - XSS protection with content security policies
  - Path traversal prevention
- **Rate Limiting Strategy**:
  - Global: 100 requests/minute per IP
  - Auth endpoints: 5 attempts/hour
  - SMS: 3 codes/hour per phone
  - API keys: 1000 requests/hour for workers
- **OWASP Compliance**: Regular security audits against OWASP Top 10

#### Infrastructure Security  
- **Network Security**: VPC isolation, security groups, WAF rules
- **Secret Management**: Environment-specific secrets, never in code
- **Monitoring & Alerting**: Real-time threat detection, anomaly alerts
- **Compliance**: GDPR-ready, China Personal Information Protection Law

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

## Technical Implementation Roadmap

### Phase 1: Infrastructure Foundation ✅
**Technologies**: Rust, Actix-web, SQLx, Redis, JWT
- Domain-driven design with clean architecture
- MySQL database with migration framework
- Redis caching layer for sessions and OTP codes
- JWT-based authentication with refresh tokens
- Comprehensive error handling and i18n support
- Audit logging and monitoring infrastructure

### Phase 2: Core Services Implementation 🚧
**Technologies**: WebSocket, Google Maps API, Elasticsearch
- Order management service with state machine
- Location service with geospatial queries
- Matching engine with weighted scoring algorithm
- Real-time notification service
- File storage service for images (S3-compatible)
- Search service with full-text and faceted search

### Phase 3: Advanced Features 📝
**Technologies**: Socket.io, Redis Pub/Sub, ML frameworks
- Real-time chat with message persistence
- Push notification service (FCM/APNS/HMS)
- Recommendation engine using collaborative filtering
- Fraud detection system with anomaly detection
- Analytics pipeline for business intelligence
- A/B testing framework for feature rollout

### Phase 4: Mobile Platform Integration 📝
**Technologies**: FFI, Swift, Kotlin, ArkTS
- Rust FFI bindings with C ABI
- Platform-specific native wrappers
- Offline-first architecture with sync
- Biometric authentication integration
- Platform-specific payment SDKs
- Deep linking and app indexing

### Phase 5: Scale & Optimization 🔒
**Technologies**: Kubernetes, Prometheus, Grafana
- Microservices migration for critical paths
- Database read replicas and sharding
- CDN integration for static assets
- GraphQL gateway for mobile optimization
- Machine learning models for pricing and matching
- International expansion with multi-region deployment