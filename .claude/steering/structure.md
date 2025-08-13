# Project Structure Steering Document - RenovEasy

## Directory Organization

```
renov-easy/
├── .claude/                     # Claude AI configuration
│   └── steering/                # Project steering documents
├── docs/                        # Documentation (date-based)
│   └── {YYYY_MM_DD}/           # Date-organized folders
│       ├── tasks/              # Task lists and TODOs
│       ├── specs/              # Requirements specifications
│       └── design/             # Design documents
├── platform/                   # Native platform implementations
│   ├── android/                # Android app (Kotlin)
│   │   ├── app/               # Main application module
│   │   ├── features/          # Feature modules
│   │   └── shared/            # Shared Android code
│   ├── ios/                    # iOS app (Swift)
│   │   ├── RenovEasy/         # Main app target
│   │   ├── Features/          # Feature modules
│   │   └── Shared/            # Shared iOS code
│   └── harmony/                # HarmonyOS app (ArkTS)
│       ├── entry/             # Entry module
│       ├── features/          # Feature modules
│       └── shared/            # Shared HarmonyOS code
├── prototype/                   # HTML/CSS prototypes
│   ├── auth/                   # Authentication flows
│   ├── styles/                 # CSS and design system
│   └── *.html                  # Page prototypes
├── services/                    # Rust shared business logic
│   ├── adapters/               # External service adapters
│   │   ├── cache/             # Caching implementations
│   │   ├── database/          # Database adapters
│   │   └── http/              # HTTP client adapters
│   ├── core/                   # Core business logic
│   │   ├── domain/            # Domain models
│   │   ├── interfaces/        # Port interfaces
│   │   └── services/          # Domain services
│   ├── ffi/                    # Foreign Function Interface
│   │   ├── android/           # Android JNI bindings
│   │   ├── harmony/           # HarmonyOS NAPI bindings
│   │   └── ios/               # iOS C bindings
│   ├── shared/                 # Shared utilities
│   │   ├── errors/            # Error definitions
│   │   ├── types/             # Common types
│   │   └── utils/             # Utility functions
│   └── tests/                  # Test suites
│       ├── integration/        # Integration tests
│       └── unit/              # Unit tests
└── tests/                       # End-to-end tests
    ├── api/                    # API tests
    ├── performance/            # Performance tests
    └── ui/                     # UI automation tests
```

## Naming Conventions

### Files
- **Rust files**: snake_case (e.g., `user_service.rs`)
- **Swift files**: PascalCase (e.g., `UserService.swift`)
- **Kotlin files**: PascalCase (e.g., `UserService.kt`)
- **TypeScript/ArkTS**: PascalCase for classes, camelCase for utilities
- **Documentation**: kebab-case (e.g., `api-design.md`)

### Code
- **Rust**: Follow Rust naming conventions
  - Modules: snake_case
  - Types/Traits: PascalCase
  - Functions/Variables: snake_case
  - Constants: SCREAMING_SNAKE_CASE
- **Swift**: Follow Swift API Design Guidelines
  - Types: PascalCase
  - Methods/Properties: camelCase
  - Constants: camelCase or PascalCase
- **Kotlin**: Follow Kotlin coding conventions
  - Classes: PascalCase
  - Functions/Variables: camelCase
  - Constants: SCREAMING_SNAKE_CASE

### API Endpoints
- **RESTful conventions**: 
  - Resources: plural nouns (`/users`, `/orders`)
  - Actions: HTTP verbs (GET, POST, PUT, DELETE)
  - Nested resources: `/users/{id}/orders`
  - Query parameters: snake_case
  - JSON fields: snake_case

## Module Organization

### Services Layer (Rust)
```
services/
├── core/
│   ├── domain/
│   │   ├── entities/          # Business entities
│   │   ├── value_objects/     # Value objects
│   │   └── events/            # Domain events
│   ├── interfaces/
│   │   ├── repositories/      # Repository interfaces
│   │   ├── services/          # Service interfaces
│   │   └── adapters/          # Adapter interfaces
│   └── services/
│       ├── auth_service.rs    # Authentication logic
│       ├── order_service.rs   # Order management
│       ├── user_service.rs    # User management
│       └── worker_service.rs  # Worker management
```

### Platform Features
Each platform follows its native conventions while maintaining consistency:

```
platform/{platform}/features/
├── auth/                       # Authentication feature
├── home/                       # Home/map feature
├── orders/                     # Order management
├── profile/                    # User profile
├── chat/                       # Messaging feature
└── common/                     # Shared components
```

## Testing Structure

### Test Organization
- **Unit tests**: Colocated with source files
- **Integration tests**: In dedicated test directories
- **E2E tests**: In root `/tests` directory

### Test Naming
- **Rust tests**: `#[test] fn should_action_when_condition()`
- **Swift tests**: `func testActionWhenCondition()`
- **Kotlin tests**: `fun \`should action when condition\`()`

## Documentation Standards

### Code Documentation
- **Rust**: Use `///` for public API documentation
- **Swift**: Use `///` or `/** */` for documentation comments
- **Kotlin**: Use KDoc format `/** */`
- **All languages**: Document public APIs, complex logic, and non-obvious decisions

### Project Documentation
- **Location**: `/docs/{YYYY_MM_DD}/` structure
- **Format**: Markdown with clear headings
- **Categories**:
  - `tasks/`: Implementation tasks and TODOs
  - `specs/`: Feature specifications
  - `design/`: Architecture and design decisions

## Git Conventions

### Branch Naming
- `main`: Production-ready code
- `develop`: Development branch
- `feature/feature-name`: New features
- `fix/bug-description`: Bug fixes
- `chore/task-description`: Maintenance tasks

### Commit Messages
- Format: `<type>(<scope>): <subject>`
- Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`
- Example: `feat(auth): add SMS verification`

## CI/CD Pipeline Structure

### Build Pipeline
1. **Lint**: Check code style
2. **Test**: Run unit and integration tests
3. **Build**: Compile all platforms
4. **Package**: Create distribution packages

### Quality Checks
- **Rust**: rustfmt, clippy
- **Swift**: SwiftLint
- **Kotlin**: ktlint
- **Coverage**: Minimum 70% for critical paths

## Configuration Management

### Environment Variables
- Development: `.env.development`
- Staging: `.env.staging`
- Production: `.env.production`

### Platform Configurations
- **Android**: `gradle.properties`, `local.properties`
- **iOS**: `.xcconfig` files
- **HarmonyOS**: `config.json`

## Security Practices

### Code Security
- Never commit secrets or API keys
- Use environment variables for sensitive data
- Implement proper input validation
- Follow OWASP mobile security guidelines

### Dependency Management
- Regular dependency updates
- Security vulnerability scanning
- License compliance checking
- Lock file maintenance

## Performance Guidelines

### Code Optimization
- Profile before optimizing
- Minimize memory allocations
- Use lazy loading where appropriate
- Implement proper caching strategies

### Resource Management
- Proper cleanup of resources
- Connection pooling for database
- Image optimization for UI assets
- Efficient data serialization