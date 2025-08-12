# Implementation Plan - User Authentication

## Task Overview
This implementation plan breaks down the user authentication feature into atomic, executable tasks. Each task is designed to be completed in 15-30 minutes by an experienced developer, focusing on specific files and clear outcomes. The tasks follow the Clean Architecture design with Rust core services and platform-specific FFI adapters.

## Steering Document Compliance
All tasks follow the structure.md conventions for file organization and tech.md patterns for Clean Architecture, Rust development standards, and FFI design. Tasks are organized to build upon the existing service structure and leverage existing adapters.

## Atomic Task Requirements
**Each task must meet these criteria for optimal agent execution:**
- **File Scope**: Touches 1-3 related files maximum
- **Time Boxing**: Completable in 15-30 minutes
- **Single Purpose**: One testable outcome per task
- **Specific Files**: Must specify exact files to create/modify
- **Agent-Friendly**: Clear input/output with minimal context switching

## Task Format Guidelines
- Use checkbox format: `- [ ] Task number. Task description`
- **Specify files**: Always include exact file paths to create/modify
- **Include implementation details** as bullet points
- Reference requirements using: `_Requirements: X.Y, Z.A_`
- Reference existing code to leverage using: `_Leverage: path/to/file.ts, path/to/component.tsx_`
- Focus only on coding tasks (no deployment, user testing, etc.)
- **Avoid broad terms**: No "system", "integration", "complete" in task titles

## Tasks

### Core Domain Models

- [ ] 1. Create User entity model in server/core/domain/entities/user.rs
  - File: server/core/domain/entities/user.rs
  - Define User struct with id, phone, country_code, email, user_type fields
  - Implement UserType enum with Customer and Worker variants
  - Add AuthMethod enum for tracking authentication methods
  - Purpose: Establish core user data structure
  - _Requirements: 1.1, 3.1_

- [ ] 2. Create UserProfile value object in server/core/domain/value_objects/user_profile.rs
  - File: server/core/domain/value_objects/user_profile.rs
  - Define UserProfile struct with name, avatar, location fields
  - Add validation methods for profile data
  - Purpose: Encapsulate user profile information
  - _Requirements: 3.2_

- [ ] 3. Create VerificationSession value object in server/core/domain/value_objects/verification.rs
  - File: server/core/domain/value_objects/verification.rs
  - Define VerificationSession struct with session_id, phone, code_hash, attempts, expires_at
  - Add methods for code validation and expiry checking
  - Purpose: Manage SMS verification state
  - _Requirements: 1.2, 1.3_

### Shared Types and Errors

- [ ] 4. Define authentication types in server/shared/types/auth_types.rs
  - File: server/shared/types/auth_types.rs
  - Create SessionToken struct with access_token, refresh_token, expires_at
  - Define SessionInfo struct with user_id, session_id, device_info
  - Add DeviceInfo struct for platform identification
  - Purpose: Share authentication types across services
  - _Requirements: 4.1_

- [ ] 5. Create authentication error definitions in server/shared/errors/auth_errors.rs
  - File: server/shared/errors/auth_errors.rs
  - Define AuthError enum with thiserror derive
  - Add error variants: InvalidPhone, SMSDeliveryFailed, InvalidCode, SessionExpired, RateLimited
  - Include error codes as constants
  - Purpose: Centralize authentication error handling
  - _Leverage: server/shared/errors/_
  - _Requirements: 1.2, 1.4_

### Phone Verification Service

- [ ] 6. Create PhoneVerification service interface in server/core/interfaces/phone_verification.rs
  - File: server/core/interfaces/phone_verification.rs
  - Define trait with send_verification_code, validate_code, resend_code methods
  - Add async trait signatures using async-trait
  - Purpose: Define phone verification contract
  - _Requirements: 1.1_

- [ ] 7. Implement PhoneVerification service in server/core/services/phone_verification.rs
  - File: server/core/services/phone_verification.rs
  - Implement verification code generation (6 digits)
  - Add code hashing with bcrypt
  - Implement rate limiting logic (5 codes per hour)
  - Purpose: Core SMS verification logic
  - _Leverage: server/adapters/http/_
  - _Requirements: 1.1, 1.2, 1.5_

- [ ] 8. Create SMS provider adapter interface in server/adapters/sms/sms_provider.rs
  - File: server/adapters/sms/sms_provider.rs
  - Define SmsProvider trait with send_sms method
  - Add SmsMessage struct with phone, message, language fields
  - Purpose: Abstract SMS provider integration
  - _Requirements: 1.1, 6.3_

- [ ] 9. Implement mock SMS provider in server/adapters/sms/mock_provider.rs
  - File: server/adapters/sms/mock_provider.rs
  - Create MockSmsProvider implementing SmsProvider trait
  - Log SMS messages to console for development
  - Add configurable delay to simulate network latency
  - Purpose: Enable testing without real SMS provider
  - _Leverage: server/adapters/sms/sms_provider.rs_
  - _Requirements: 1.1_

### Session Management

- [ ] 10. Create SessionManager service in server/core/services/session_manager.rs
  - File: server/core/services/session_manager.rs
  - Implement JWT token generation with jsonwebtoken crate
  - Add create_session method with 1-hour access token expiry
  - Implement refresh_token method with 30-day refresh token
  - Purpose: Manage authentication sessions
  - _Requirements: 4.1, 4.2_

- [ ] 11. Add token validation to SessionManager in server/core/services/session_manager.rs
  - File: server/core/services/session_manager.rs (modify)
  - Implement validate_token method with JWT verification
  - Add token expiry checking
  - Include device fingerprint validation
  - Purpose: Secure token validation
  - _Requirements: 4.2, 4.4_

- [ ] 12. Create cache adapter for sessions in server/adapters/cache/session_cache.rs
  - File: server/adapters/cache/session_cache.rs
  - Define SessionCache trait with get, set, invalidate methods
  - Add TTL support for automatic expiry
  - Purpose: Enable fast session validation
  - _Leverage: server/adapters/cache/_
  - _Requirements: 4.1_

### Social Authentication

- [ ] 13. Create OAuth provider interface in server/core/interfaces/oauth_provider.rs
  - File: server/core/interfaces/oauth_provider.rs
  - Define OAuthProvider trait with initiate_oauth, handle_callback methods
  - Add OAuthSession struct for state management
  - Purpose: Abstract OAuth provider integration
  - _Requirements: 2.1_

- [ ] 14. Implement SocialAuth service in server/core/services/social_auth.rs
  - File: server/core/services/social_auth.rs
  - Create service managing multiple OAuth providers
  - Implement PKCE flow for enhanced security
  - Add provider registration and selection logic
  - Purpose: Orchestrate social authentication
  - _Requirements: 2.1, 2.2_

- [ ] 15. Create OAuth state manager in server/core/services/oauth_state.rs
  - File: server/core/services/oauth_state.rs
  - Implement secure state generation and validation
  - Add 10-minute expiry for OAuth states
  - Store states in cache with TTL
  - Purpose: Prevent CSRF in OAuth flow
  - _Leverage: server/adapters/cache/_
  - _Requirements: 2.1, 2.5_

### User Type Management

- [ ] 16. Create UserTypeManager service in server/core/services/user_type_manager.rs
  - File: server/core/services/user_type_manager.rs
  - Implement set_user_type method updating user profile
  - Add get_user_permissions returning role-based permissions
  - Include worker verification deadline logic (30 days)
  - Purpose: Handle user role configuration
  - _Requirements: 3.1, 3.2, 3.3_

- [ ] 17. Define permission system in server/core/domain/value_objects/permissions.rs
  - File: server/core/domain/value_objects/permissions.rs
  - Create Permission enum with platform capabilities
  - Add PermissionSet struct for role-based access
  - Purpose: Enable role-based access control
  - _Requirements: 3.2, 3.3_

### Core Authentication Service

- [ ] 18. Create AuthService orchestrator in server/core/services/auth_service.rs
  - File: server/core/services/auth_service.rs
  - Define main AuthService struct with dependencies
  - Implement authenticate_with_phone initiating SMS flow
  - Add verify_code completing phone authentication
  - Purpose: Coordinate authentication operations
  - _Leverage: server/core/services/phone_verification.rs, services/core/services/session_manager.rs_
  - _Requirements: 1.1, 1.3_

- [ ] 19. Add social auth methods to AuthService in server/core/services/auth_service.rs
  - File: server/core/services/auth_service.rs (modify)
  - Implement authenticate_with_social for OAuth flow
  - Add account linking for existing users
  - Include new user registration via social auth
  - Purpose: Integrate social authentication
  - _Leverage: server/core/services/social_auth.rs_
  - _Requirements: 2.1, 2.2, 2.3_

- [ ] 20. Implement logout in AuthService in server/core/services/auth_service.rs
  - File: server/core/services/auth_service.rs (modify)
  - Add logout method invalidating all user sessions
  - Clear cached session data
  - Notify other devices of logout (optional)
  - Purpose: Secure session termination
  - _Leverage: server/core/services/session_manager.rs_
  - _Requirements: 4.3_

### Database Integration

- [ ] 21. Create user repository interface in server/core/interfaces/repositories/user_repository.rs
  - File: server/core/interfaces/repositories/user_repository.rs
  - Define UserRepository trait with CRUD operations
  - Add find_by_phone and find_by_social_id methods
  - Purpose: Abstract user data persistence
  - _Requirements: 1.3, 2.2_

- [ ] 22. Implement MySQL user repository in server/adapters/database/mysql/user_repository_impl.rs
  - File: server/adapters/database/mysql/user_repository_impl.rs
  - Implement UserRepository trait using sqlx
  - Add database migrations for users table
  - Include phone number encryption at rest
  - Purpose: Persist user data in MySQL
  - _Leverage: server/adapters/database/_
  - _Requirements: 1.3, 2.2_

- [ ] 23. Create session repository in server/adapters/database/mysql/session_repository_impl.rs
  - File: server/adapters/database/mysql/session_repository_impl.rs
  - Implement session storage and retrieval
  - Add device tracking for multi-device support
  - Include session invalidation queries
  - Purpose: Persist session data
  - _Leverage: server/adapters/database/_
  - _Requirements: 4.1, 5.1_

### FFI Bridge - Common

- [ ] 24. Create C-compatible auth types in server/ffi/common/auth_types.h
  - File: server/ffi/common/auth_types.h
  - Define C structs for AuthRequest, AuthResponse, SessionToken
  - Add error code constants matching Rust errors
  - Purpose: Share types across FFI boundaries
  - _Requirements: 5.2_

- [ ] 25. Implement FFI error mapping in server/ffi/common/error_mapping.rs
  - File: server/ffi/common/error_mapping.rs
  - Create error code conversion from Rust to C integers
  - Add error message serialization
  - Purpose: Consistent error handling across platforms
  - _Leverage: server/shared/errors/auth_errors.rs_
  - _Requirements: 5.2_

### FFI Bridge - iOS

- [ ] 26. Create iOS FFI bridge in server/ffi/ios/auth_bridge.rs
  - File: server/ffi/ios/auth_bridge.rs
  - Export C functions: auth_phone_send_code, auth_phone_verify
  - Implement JSON serialization for Swift interop
  - Add proper memory management with Box and CString
  - Purpose: Enable iOS app integration
  - _Leverage: server/core/services/auth_service.rs_
  - _Requirements: 5.2_

- [ ] 27. Add iOS session management FFI in server/ffi/ios/session_bridge.rs
  - File: server/ffi/ios/session_bridge.rs
  - Export functions: auth_validate_token, auth_refresh_token
  - Handle Swift callback for async operations
  - Purpose: iOS session management
  - _Leverage: server/core/services/session_manager.rs_
  - _Requirements: 4.1, 5.2_

### FFI Bridge - Android

- [ ] 28. Create Android JNI bridge in server/ffi/android/auth_jni.rs
  - File: server/ffi/android/auth_jni.rs
  - Implement JNI functions using jni crate
  - Add Java class generation annotations
  - Handle JNI local/global references properly
  - Purpose: Enable Android app integration
  - _Leverage: server/core/services/auth_service.rs_
  - _Requirements: 5.2_

- [ ] 29. Add Android session JNI in server/ffi/android/session_jni.rs
  - File: server/ffi/android/session_jni.rs
  - Implement session validation and refresh JNI methods
  - Add Kotlin coroutine support for async calls
  - Purpose: Android session management
  - _Leverage: server/core/services/session_manager.rs_
  - _Requirements: 4.1, 5.2_

### FFI Bridge - HarmonyOS

- [ ] 30. Create HarmonyOS NAPI bridge in server/ffi/harmony/auth_napi.rs
  - File: server/ffi/harmony/auth_napi.rs
  - Implement NAPI bindings for ArkTS
  - Add Promise-based async function exports
  - Handle JavaScript type conversions
  - Purpose: Enable HarmonyOS app integration
  - _Leverage: server/core/services/auth_service.rs_
  - _Requirements: 5.2_

### API Layer

- [ ] 31. Create authentication REST controller in server/api/controllers/auth_controller.rs
  - File: server/api/controllers/auth_controller.rs
  - Define HTTP handlers for phone authentication endpoints
  - Add request validation middleware
  - Implement proper HTTP status codes
  - Purpose: Expose authentication via REST API
  - _Leverage: server/core/services/auth_service.rs_
  - _Requirements: 1.1_

- [ ] 32. Add social auth endpoints in server/api/controllers/social_auth_controller.rs
  - File: server/api/controllers/social_auth_controller.rs
  - Implement OAuth initiation and callback handlers
  - Add CSRF protection with state validation
  - Purpose: Handle OAuth flow via API
  - _Leverage: server/core/services/social_auth.rs_
  - _Requirements: 2.1_

- [ ] 33. Create session management endpoints in server/api/controllers/session_controller.rs
  - File: server/api/controllers/session_controller.rs
  - Implement token refresh endpoint
  - Add logout endpoint invalidating sessions
  - Include rate limiting middleware
  - Purpose: Manage sessions via API
  - _Leverage: server/core/services/session_manager.rs_
  - _Requirements: 4.2, 4.3_

### Configuration

- [ ] 34. Add authentication config in server/shared/config/auth_config.rs
  - File: server/shared/config/auth_config.rs
  - Define configuration structs for JWT, SMS, OAuth settings
  - Load from environment variables
  - Add validation for required settings
  - Purpose: Centralize authentication configuration
  - _Leverage: server/shared/config/_
  - _Requirements: 1.1, 2.1, 4.1_

### Testing - Unit Tests

- [ ] 35. Write User entity tests in server/core/domain/entities/user_test.rs
  - File: server/core/domain/entities/user_test.rs
  - Test User struct creation and validation
  - Verify UserType enum behavior
  - Test AuthMethod serialization
  - Purpose: Ensure domain model correctness
  - _Requirements: 3.1_

- [ ] 36. Write PhoneVerification service tests in server/core/services/phone_verification_test.rs
  - File: server/core/services/phone_verification_test.rs
  - Test code generation and validation
  - Verify rate limiting enforcement
  - Test code expiry logic
  - Purpose: Validate SMS verification logic
  - _Requirements: 1.1, 1.4, 1.5_

- [ ] 37. Write SessionManager tests in server/core/services/session_manager_test.rs
  - File: server/core/services/session_manager_test.rs
  - Test JWT token generation and validation
  - Verify token refresh logic
  - Test session invalidation
  - Purpose: Ensure session security
  - _Requirements: 4.1, 4.2_

### Testing - Integration Tests

- [ ] 38. Create phone auth flow integration test in server/tests/integration/phone_auth_test.rs
  - File: server/tests/integration/phone_auth_test.rs
  - Test complete phone authentication flow
  - Verify database persistence
  - Test error scenarios
  - Purpose: Validate end-to-end phone auth
  - _Requirements: 1.1, 1.3_

- [ ] 39. Create multi-platform session test in server/tests/integration/cross_platform_test.rs
  - File: server/tests/integration/cross_platform_test.rs
  - Test session sharing across platforms
  - Verify token validation consistency
  - Test device tracking
  - Purpose: Ensure cross-platform compatibility
  - _Requirements: 5.1, 5.2_

### Internationalization

- [ ] 40. Create i18n message files in server/shared/i18n/auth_messages.rs
  - File: server/shared/i18n/auth_messages.rs
  - Define message keys for authentication texts
  - Add Chinese and English translations
  - Include SMS template messages
  - Purpose: Support bilingual authentication
  - _Requirements: 6.1, 6.2, 6.3_