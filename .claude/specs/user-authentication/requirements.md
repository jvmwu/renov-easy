# Requirements Document - User Authentication

## Introduction

The user authentication system is a core feature of the RenovEasy application, providing secure and convenient user identity verification and authorization mechanisms for the renovation services marketplace platform. The system adopts a passwordless authentication method based on phone numbers, completing user registration and login through SMS verification codes, and supporting differentiated management of two user roles: customers and workers.

## Alignment with Product Vision

This feature directly supports the core value propositions defined in product.md:
- **Convenient Communication**: Quick registration through phone number, lowering user entry barriers
- **Bilingual Support**: Authentication flow supports Chinese and English, meeting Australian market needs
- **User Classification**: Distinguish between customer and worker roles, laying the foundation for subsequent service matching
- **Trust Building**: Ensure user authenticity through phone verification, establishing platform trust foundation

## Requirements

### Requirement 1: Phone Number Registration and Login

**User Story:** As a new user, I want to register and login using my phone number, so that I can quickly access the platform without remembering passwords.

#### Acceptance Criteria

1. WHEN user enters a valid phone number THEN system SHALL send a 6-digit verification code via SMS
2. IF phone number format is invalid THEN system SHALL display appropriate error message
3. WHEN SMS is sent successfully THEN system SHALL display verification code input screen
4. IF SMS sending fails THEN system SHALL provide fallback options and error recovery
5. WHEN user is on verification screen THEN system SHALL display 60-second countdown for resend

### Requirement 2: SMS Verification

**User Story:** As a user, I want to verify my phone number with an SMS code, so that my identity can be confirmed securely.

#### Acceptance Criteria

1. WHEN user enters correct 6-digit code THEN system SHALL verify and proceed to next step
2. IF verification code is incorrect THEN system SHALL display error and allow retry (max 3 attempts)
3. WHEN verification code expires (5 minutes) THEN system SHALL require new code request
4. IF user pastes verification code THEN system SHALL auto-fill all 6 digits
5. WHEN 60-second countdown expires THEN system SHALL enable resend button

### Requirement 3: User Type Selection

**User Story:** As a new user, I want to select whether I'm a customer or worker, so that I can access appropriate features for my role.

#### Acceptance Criteria

1. WHEN new user completes phone verification THEN system SHALL present user type selection
2. IF user selects "Customer" THEN system SHALL create customer profile and redirect to customer home
3. IF user selects "Worker" THEN system SHALL create worker profile and redirect to worker home
4. WHEN existing user logs in THEN system SHALL skip type selection and use stored type
5. IF worker is selected THEN system SHALL flag account for professional verification requirement

### Requirement 4: JWT Token Management

**User Story:** As an authenticated user, I want my session to remain secure and persistent, so that I don't need to login repeatedly while maintaining security.

#### Acceptance Criteria

1. WHEN user successfully authenticates THEN system SHALL issue JWT access token (15-minute expiry)
2. WHEN user successfully authenticates THEN system SHALL issue refresh token (7-day expiry)
3. IF access token expires THEN system SHALL automatically refresh using refresh token
4. WHEN user logs out THEN system SHALL invalidate both access and refresh tokens
5. IF refresh token expires THEN system SHALL require user to re-authenticate

### Requirement 5: Rate Limiting and Security

**User Story:** As a platform operator, I want to prevent abuse of the authentication system, so that the service remains secure and cost-effective.

#### Acceptance Criteria

1. WHEN user requests SMS code THEN system SHALL limit to 3 requests per phone per hour
2. IF rate limit is exceeded THEN system SHALL block further requests with clear error message
3. WHEN verification fails 3 times THEN system SHALL temporarily lock the phone number (30 minutes)
4. IF suspicious activity detected THEN system SHALL trigger additional verification requirements
5. WHEN API is called THEN system SHALL enforce HTTPS and validate request origins

### Requirement 6: Multi-Platform Support

**User Story:** As a mobile app user, I want to authenticate seamlessly across iOS, Android, and HarmonyOS, so that I have consistent experience regardless of platform.

#### Acceptance Criteria

1. WHEN authentication API is called from any platform THEN system SHALL provide consistent response format
2. IF platform-specific token storage is needed THEN system SHALL provide FFI bindings
3. WHEN user switches platforms THEN system SHALL maintain session continuity
4. IF platform has biometric capability THEN system SHALL support biometric unlock after initial login
5. WHEN offline THEN system SHALL gracefully handle authentication state

## Non-Functional Requirements

### Performance
- SMS delivery SHALL complete within 30 seconds for 95% of requests
- Authentication API response time SHALL be less than 200ms for login/verification
- Token refresh SHALL complete within 100ms
- System SHALL support 1000 concurrent authentication requests

### Security
- All authentication endpoints SHALL use TLS 1.3 encryption
- Verification codes SHALL be cryptographically random 6-digit numbers
- JWT secrets SHALL be rotated monthly
- Failed authentication attempts SHALL be logged for security monitoring
- Phone numbers SHALL be hashed when stored in database

### Reliability
- Authentication service SHALL maintain 99.9% uptime
- SMS delivery SHALL have fallback provider for redundancy
- Database SHALL use connection pooling with automatic reconnection
- Token validation SHALL work offline using cached public keys

### Usability
- Phone number input SHALL support international formats with country code selection
- Error messages SHALL be clear and actionable in both Chinese and English
- Verification code input SHALL support paste and auto-progression
- Loading states SHALL provide visual feedback for all async operations
- Success states SHALL clearly indicate next steps
- 