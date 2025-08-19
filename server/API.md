# RenovEasy API Documentation

## Table of Contents
- [Overview](#overview)
- [Base URL](#base-url)
- [API Version](#api-version)
- [Authentication](#authentication)
- [Rate Limiting](#rate-limiting)
- [Request/Response Format](#requestresponse-format)
- [Error Handling](#error-handling)
- [Endpoints](#endpoints)
  - [Health Check](#health-check)
  - [Authentication Endpoints](#authentication-endpoints)
    - [Send Verification Code](#send-verification-code)
    - [Verify Code](#verify-code)
    - [Select User Type](#select-user-type)
    - [Refresh Token](#refresh-token)
    - [Logout](#logout)
- [Error Codes Reference](#error-codes-reference)
- [Security Considerations](#security-considerations)
- [Examples](#examples)

## Overview

The RenovEasy API provides a secure, phone number-based passwordless authentication system for the RenovEasy platform. It supports two user types (customers and workers) and provides bilingual support for Chinese and English.

### Key Features
- Passwordless authentication via SMS verification
- JWT-based token management
- User type selection (customer/worker)
- Bilingual error messages (Chinese/English)
- Comprehensive rate limiting
- Security headers and CORS support

## Base URL

```
Production: https://api.renoveasy.com
Development: http://localhost:8080
```

## API Version

Current version: `v1`

All API endpoints are prefixed with `/api/v1`

## Authentication

The API uses JWT (JSON Web Tokens) for authentication. The authentication flow is:

1. **Request verification code**: Call `/auth/send-code` with phone number
2. **Verify code**: Call `/auth/verify-code` with phone and code
3. **Select user type** (if new user): Call `/auth/select-type` with user type
4. **Use access token**: Include token in `Authorization` header for protected endpoints
5. **Refresh token**: Use refresh token to get new access token when expired

### Authorization Header

For protected endpoints, include the JWT token in the Authorization header:

```
Authorization: Bearer <access_token>
```

### Token Lifecycle
- **Access Token**: Valid for 15 minutes
- **Refresh Token**: Valid for 30 days
- Tokens contain user ID, phone number, and user type (once selected)

## Rate Limiting

The API implements multiple layers of rate limiting to prevent abuse:

### SMS Rate Limits
| Limit Type | Value | Duration |
|------------|-------|----------|
| Per phone per hour | 3 | 1 hour |
| Per phone per day | 10 | 24 hours |
| Verification attempts per code | 3 | Per code |
| Cooldown between SMS | 60 seconds | Between requests |

### API Rate Limits
| Limit Type | Value | Duration |
|------------|-------|----------|
| Per IP per minute | 60 | 1 minute |
| Per IP per hour | 1000 | 1 hour |
| Per user per minute | 100 | 1 minute |
| Per user per hour | 2000 | 1 hour |
| Burst limit | 10 | 1 second |

### Authentication Rate Limits
| Limit Type | Value | Duration |
|------------|-------|----------|
| Login per IP per hour | 10 | 1 hour |
| Login per user per hour | 5 | 1 hour |
| Failed attempts threshold | 5 | Before lock |
| Account lock duration | 30 minutes | After threshold |

## Request/Response Format

### Request Headers

All requests should include:

```http
Content-Type: application/json
Accept-Language: zh-CN  # or 'en' for English
```

### Response Format

All responses follow a consistent JSON structure:

#### Success Response
```json
{
  "data": {
    // Response data
  },
  "success": true
}
```

#### Error Response
```json
{
  "error": {
    "code": "error_code",
    "message": "Human-readable error message",
    "details": {}  // Optional additional error details
  },
  "success": false
}
```

## Error Handling

The API uses standard HTTP status codes and provides detailed error information in the response body.

### HTTP Status Codes

| Status Code | Description |
|-------------|-------------|
| 200 | Success |
| 400 | Bad Request - Invalid input |
| 401 | Unauthorized - Invalid or missing token |
| 403 | Forbidden - Access denied |
| 404 | Not Found - Resource not found |
| 429 | Too Many Requests - Rate limit exceeded |
| 500 | Internal Server Error |
| 503 | Service Unavailable |

## Endpoints

### Health Check

Check the API service status.

```http
GET /health
```

#### Response
```json
{
  "status": "healthy",
  "service": "renov-easy-api",
  "version": "0.1.0",
  "timestamp": "2025-01-19T10:00:00Z"
}
```

### Authentication Endpoints

#### Send Verification Code

Send SMS verification code to a phone number.

```http
POST /api/v1/auth/send-code
```

##### Request Body
```json
{
  "phone": "1234567890",
  "country_code": "+1"
}
```

##### Validation Rules
- `phone`: 10-15 characters, numeric
- `country_code`: 1-10 characters

##### Success Response (200)
```json
{
  "message": "Verification code sent successfully",
  "resend_after": 60
}
```

##### Error Responses

**400 Bad Request**
```json
{
  "error": {
    "code": "invalid_phone_format",
    "message": "Invalid phone number format: 123"
  }
}
```

**429 Too Many Requests**
```json
{
  "error": {
    "code": "rate_limit_exceeded",
    "message": "Too many requests. Please try again in 5 minutes"
  }
}
```

**503 Service Unavailable**
```json
{
  "error": {
    "code": "sms_service_failure",
    "message": "SMS service is temporarily unavailable. Please try again later"
  }
}
```

#### Verify Code

Verify the SMS code and authenticate the user.

```http
POST /api/v1/auth/verify-code
```

##### Request Body
```json
{
  "phone": "1234567890",
  "country_code": "+1",
  "code": "123456"
}
```

##### Validation Rules
- `phone`: 10-15 characters, numeric
- `country_code`: 1-10 characters
- `code`: Exactly 6 characters

##### Success Response (200)

For existing users with selected type:
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
  "expires_in": 900,
  "user_type": "customer",
  "requires_type_selection": false
}
```

For new users (need to select type):
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
  "expires_in": 900,
  "user_type": null,
  "requires_type_selection": true
}
```

##### Error Responses

**400 Bad Request**
```json
{
  "error": {
    "code": "invalid_verification_code",
    "message": "Invalid or expired verification code"
  }
}
```

**403 Forbidden**
```json
{
  "error": {
    "code": "user_blocked",
    "message": "User account has been blocked"
  }
}
```

**429 Too Many Requests**
```json
{
  "error": {
    "code": "max_attempts_exceeded",
    "message": "Maximum verification attempts exceeded. Please request a new code"
  }
}
```

#### Select User Type

Select user type (customer or worker) for new users.

```http
POST /api/v1/auth/select-type
```

**Authorization Required**: Yes

##### Request Headers
```http
Authorization: Bearer <access_token>
```

##### Request Body
```json
{
  "user_type": "customer"
}
```

##### Valid User Types
- `customer` - Service customer
- `worker` - Service provider/worker

##### Success Response (200)
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
  "expires_in": 900,
  "user_type": "customer",
  "requires_type_selection": false
}
```

##### Error Responses

**400 Bad Request**
```json
{
  "error": {
    "code": "validation_error",
    "message": "Invalid user type. Must be 'customer' or 'worker'"
  }
}
```

**401 Unauthorized**
```json
{
  "error": {
    "code": "authentication_failed",
    "message": "Authentication failed"
  }
}
```

**403 Forbidden**
```json
{
  "error": {
    "code": "business_rule_violation",
    "message": "User type has already been selected"
  }
}
```

#### Refresh Token

Refresh an expired access token using a refresh token.

```http
POST /api/v1/auth/refresh
```

##### Request Body
```json
{
  "refresh_token": "eyJhbGciOiJIUzI1NiIs..."
}
```

##### Success Response (200)
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
  "expires_in": 900,
  "user_type": "customer",
  "requires_type_selection": false
}
```

##### Error Responses

**401 Unauthorized**
```json
{
  "error": {
    "code": "invalid_refresh_token",
    "message": "Invalid refresh token"
  }
}
```

**401 Unauthorized**
```json
{
  "error": {
    "code": "refresh_token_expired",
    "message": "Refresh token has expired"
  }
}
```

#### Logout

Logout and invalidate the current session tokens.

```http
POST /api/v1/auth/logout
```

**Authorization Required**: Yes

##### Request Headers
```http
Authorization: Bearer <access_token>
```

##### Success Response (200)
```json
{
  "message": "Successfully logged out"
}
```

##### Error Responses

**401 Unauthorized**
```json
{
  "error": {
    "code": "authentication_failed",
    "message": "Authentication failed"
  }
}
```

## Error Codes Reference

### Authentication Errors

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `invalid_phone_format` | 400 | Invalid phone number format |
| `rate_limit_exceeded` | 429 | Too many requests |
| `sms_service_failure` | 503 | SMS service unavailable |
| `invalid_verification_code` | 400 | Invalid or expired verification code |
| `verification_code_expired` | 400 | Verification code has expired |
| `max_attempts_exceeded` | 429 | Maximum verification attempts exceeded |
| `user_not_found` | 404 | User not found |
| `user_already_exists` | 409 | User already exists |
| `authentication_failed` | 401 | Authentication failed |
| `insufficient_permissions` | 403 | Insufficient permissions |
| `account_suspended` | 403 | Account has been suspended |
| `session_expired` | 401 | Session has expired |
| `registration_disabled` | 503 | Registration is currently disabled |
| `user_blocked` | 403 | User account has been blocked |

### Validation Errors

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `required_field` | 400 | Required field missing |
| `invalid_format` | 400 | Invalid format for field |
| `out_of_range` | 400 | Field value out of range |
| `invalid_length` | 400 | Invalid field length |
| `pattern_mismatch` | 400 | Pattern mismatch for field |
| `invalid_email` | 400 | Invalid email format |
| `invalid_url` | 400 | Invalid URL format |
| `invalid_date` | 400 | Invalid date format |
| `duplicate_value` | 409 | Duplicate value for field |
| `business_rule_violation` | 400 | Business rule violation |

### Token Errors

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `token_expired` | 401 | Token has expired |
| `invalid_token_format` | 401 | Invalid token format |
| `invalid_signature` | 401 | Invalid token signature |
| `token_not_yet_valid` | 401 | Token is not yet valid |
| `invalid_claims` | 401 | Invalid token claims |
| `token_revoked` | 401 | Token has been revoked |
| `refresh_token_expired` | 401 | Refresh token has expired |
| `invalid_refresh_token` | 401 | Invalid refresh token |
| `token_generation_failed` | 500 | Failed to generate token |
| `missing_claim` | 400 | Missing required claim |

### General Errors

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `validation_error` | 400 | General validation error |
| `business_rule_violation` | 400 | Business rule violation |
| `not_found` | 404 | Resource not found |
| `unauthorized` | 401 | Unauthorized access |
| `internal_error` | 500 | Internal server error |

## Security Considerations

### Security Headers

The API automatically includes the following security headers:

```http
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Content-Security-Policy: default-src 'none'
Strict-Transport-Security: max-age=31536000; includeSubDomains
```

### CORS Configuration

The API supports CORS with the following configuration:
- Allowed Origins: Configured per environment
- Allowed Methods: GET, POST, PUT, DELETE, OPTIONS
- Allowed Headers: Content-Type, Authorization, Accept-Language
- Credentials: Supported

### Best Practices

1. **Always use HTTPS** in production
2. **Store tokens securely** (use secure storage on mobile devices)
3. **Implement token refresh** before expiration
4. **Handle rate limits gracefully** with exponential backoff
5. **Validate all inputs** on the client side before sending
6. **Never log sensitive data** like tokens or verification codes
7. **Implement proper error handling** for all API calls

## Examples

### cURL Examples

#### Send Verification Code
```bash
curl -X POST https://api.renoveasy.com/api/v1/auth/send-code \
  -H "Content-Type: application/json" \
  -H "Accept-Language: en" \
  -d '{
    "phone": "1234567890",
    "country_code": "+1"
  }'
```

#### Verify Code
```bash
curl -X POST https://api.renoveasy.com/api/v1/auth/verify-code \
  -H "Content-Type: application/json" \
  -H "Accept-Language: en" \
  -d '{
    "phone": "1234567890",
    "country_code": "+1",
    "code": "123456"
  }'
```

#### Select User Type
```bash
curl -X POST https://api.renoveasy.com/api/v1/auth/select-type \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIs..." \
  -H "Accept-Language: en" \
  -d '{
    "user_type": "customer"
  }'
```

#### Refresh Token
```bash
curl -X POST https://api.renoveasy.com/api/v1/auth/refresh \
  -H "Content-Type: application/json" \
  -H "Accept-Language: en" \
  -d '{
    "refresh_token": "eyJhbGciOiJIUzI1NiIs..."
  }'
```

#### Logout
```bash
curl -X POST https://api.renoveasy.com/api/v1/auth/logout \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIs..." \
  -H "Accept-Language: en"
```

### JavaScript (Fetch API) Examples

#### Send Verification Code
```javascript
async function sendVerificationCode(phone, countryCode) {
  const response = await fetch('https://api.renoveasy.com/api/v1/auth/send-code', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Accept-Language': 'en'
    },
    body: JSON.stringify({
      phone: phone,
      country_code: countryCode
    })
  });
  
  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error.message);
  }
  
  return await response.json();
}
```

#### Verify Code with Error Handling
```javascript
async function verifyCode(phone, countryCode, code) {
  try {
    const response = await fetch('https://api.renoveasy.com/api/v1/auth/verify-code', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Accept-Language': 'en'
      },
      body: JSON.stringify({
        phone: phone,
        country_code: countryCode,
        code: code
      })
    });
    
    const data = await response.json();
    
    if (!response.ok) {
      // Handle specific error codes
      switch (data.error.code) {
        case 'invalid_verification_code':
          console.error('Invalid code entered');
          break;
        case 'max_attempts_exceeded':
          console.error('Too many attempts, request new code');
          break;
        case 'user_blocked':
          console.error('Account is blocked');
          break;
        default:
          console.error(data.error.message);
      }
      throw new Error(data.error.message);
    }
    
    // Store tokens securely
    localStorage.setItem('access_token', data.access_token);
    localStorage.setItem('refresh_token', data.refresh_token);
    
    // Check if user needs to select type
    if (data.requires_type_selection) {
      // Redirect to type selection
      window.location.href = '/select-type';
    }
    
    return data;
  } catch (error) {
    console.error('Verification failed:', error);
    throw error;
  }
}
```

#### Auto-refresh Token
```javascript
class TokenManager {
  constructor() {
    this.refreshTimeout = null;
  }
  
  async refreshToken() {
    const refreshToken = localStorage.getItem('refresh_token');
    
    if (!refreshToken) {
      throw new Error('No refresh token available');
    }
    
    const response = await fetch('https://api.renoveasy.com/api/v1/auth/refresh', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Accept-Language': 'en'
      },
      body: JSON.stringify({
        refresh_token: refreshToken
      })
    });
    
    if (!response.ok) {
      // Refresh failed, redirect to login
      localStorage.clear();
      window.location.href = '/login';
      throw new Error('Token refresh failed');
    }
    
    const data = await response.json();
    
    // Update stored tokens
    localStorage.setItem('access_token', data.access_token);
    localStorage.setItem('refresh_token', data.refresh_token);
    
    // Schedule next refresh (5 minutes before expiry)
    this.scheduleRefresh(data.expires_in);
    
    return data.access_token;
  }
  
  scheduleRefresh(expiresIn) {
    // Clear any existing timeout
    if (this.refreshTimeout) {
      clearTimeout(this.refreshTimeout);
    }
    
    // Schedule refresh 5 minutes before token expires
    const refreshIn = (expiresIn - 300) * 1000;
    this.refreshTimeout = setTimeout(() => {
      this.refreshToken();
    }, refreshIn);
  }
  
  async makeAuthenticatedRequest(url, options = {}) {
    const accessToken = localStorage.getItem('access_token');
    
    const response = await fetch(url, {
      ...options,
      headers: {
        ...options.headers,
        'Authorization': `Bearer ${accessToken}`,
        'Content-Type': 'application/json',
        'Accept-Language': 'en'
      }
    });
    
    // If token expired, try to refresh and retry
    if (response.status === 401) {
      const newToken = await this.refreshToken();
      
      return fetch(url, {
        ...options,
        headers: {
          ...options.headers,
          'Authorization': `Bearer ${newToken}`,
          'Content-Type': 'application/json',
          'Accept-Language': 'en'
        }
      });
    }
    
    return response;
  }
}
```

### Python Examples

#### Complete Authentication Flow
```python
import requests
import time
import json
from typing import Optional, Dict, Any

class RenovEasyAPI:
    def __init__(self, base_url: str = "https://api.renoveasy.com", language: str = "en"):
        self.base_url = base_url
        self.language = language
        self.access_token: Optional[str] = None
        self.refresh_token: Optional[str] = None
        self.token_expires_at: Optional[float] = None
        
    def _make_request(self, method: str, endpoint: str, 
                     data: Optional[Dict] = None, 
                     authenticated: bool = False) -> Dict[str, Any]:
        """Make HTTP request to API"""
        url = f"{self.base_url}{endpoint}"
        headers = {
            "Content-Type": "application/json",
            "Accept-Language": self.language
        }
        
        if authenticated and self.access_token:
            headers["Authorization"] = f"Bearer {self.access_token}"
        
        response = requests.request(
            method=method,
            url=url,
            json=data,
            headers=headers
        )
        
        # Handle token expiration
        if response.status_code == 401 and authenticated:
            self.refresh_access_token()
            headers["Authorization"] = f"Bearer {self.access_token}"
            response = requests.request(
                method=method,
                url=url,
                json=data,
                headers=headers
            )
        
        response_data = response.json()
        
        if not response.ok:
            error = response_data.get("error", {})
            raise APIError(
                code=error.get("code"),
                message=error.get("message"),
                status_code=response.status_code
            )
        
        return response_data
    
    def send_verification_code(self, phone: str, country_code: str) -> Dict[str, Any]:
        """Send SMS verification code"""
        return self._make_request(
            method="POST",
            endpoint="/api/v1/auth/send-code",
            data={
                "phone": phone,
                "country_code": country_code
            }
        )
    
    def verify_code(self, phone: str, country_code: str, code: str) -> Dict[str, Any]:
        """Verify SMS code and authenticate"""
        response = self._make_request(
            method="POST",
            endpoint="/api/v1/auth/verify-code",
            data={
                "phone": phone,
                "country_code": country_code,
                "code": code
            }
        )
        
        # Store tokens
        self.access_token = response["access_token"]
        self.refresh_token = response["refresh_token"]
        self.token_expires_at = time.time() + response["expires_in"]
        
        return response
    
    def select_user_type(self, user_type: str) -> Dict[str, Any]:
        """Select user type (customer or worker)"""
        response = self._make_request(
            method="POST",
            endpoint="/api/v1/auth/select-type",
            data={"user_type": user_type},
            authenticated=True
        )
        
        # Update tokens
        self.access_token = response["access_token"]
        self.refresh_token = response["refresh_token"]
        self.token_expires_at = time.time() + response["expires_in"]
        
        return response
    
    def refresh_access_token(self) -> Dict[str, Any]:
        """Refresh the access token"""
        if not self.refresh_token:
            raise ValueError("No refresh token available")
        
        response = self._make_request(
            method="POST",
            endpoint="/api/v1/auth/refresh",
            data={"refresh_token": self.refresh_token}
        )
        
        # Update tokens
        self.access_token = response["access_token"]
        self.refresh_token = response["refresh_token"]
        self.token_expires_at = time.time() + response["expires_in"]
        
        return response
    
    def logout(self) -> Dict[str, Any]:
        """Logout and invalidate tokens"""
        response = self._make_request(
            method="POST",
            endpoint="/api/v1/auth/logout",
            authenticated=True
        )
        
        # Clear tokens
        self.access_token = None
        self.refresh_token = None
        self.token_expires_at = None
        
        return response
    
    def is_token_expired(self) -> bool:
        """Check if access token is expired"""
        if not self.token_expires_at:
            return True
        # Consider token expired 1 minute before actual expiry
        return time.time() >= (self.token_expires_at - 60)

class APIError(Exception):
    def __init__(self, code: str, message: str, status_code: int):
        self.code = code
        self.message = message
        self.status_code = status_code
        super().__init__(f"{code}: {message}")

# Usage example
if __name__ == "__main__":
    api = RenovEasyAPI()
    
    try:
        # Step 1: Send verification code
        print("Sending verification code...")
        result = api.send_verification_code(
            phone="1234567890",
            country_code="+1"
        )
        print(f"Code sent. Can resend after {result['resend_after']} seconds")
        
        # Step 2: Get code from user (simulate)
        code = input("Enter verification code: ")
        
        # Step 3: Verify code
        print("Verifying code...")
        auth_result = api.verify_code(
            phone="1234567890",
            country_code="+1",
            code=code
        )
        
        print(f"Authentication successful!")
        print(f"User type: {auth_result.get('user_type', 'Not selected')}")
        
        # Step 4: Select user type if needed
        if auth_result["requires_type_selection"]:
            print("Please select user type...")
            type_result = api.select_user_type("customer")
            print(f"User type selected: {type_result['user_type']}")
        
        # Step 5: Make authenticated requests
        # ... your authenticated API calls here ...
        
        # Step 6: Logout when done
        api.logout()
        print("Logged out successfully")
        
    except APIError as e:
        print(f"API Error: {e.message} (Code: {e.code})")
        
        # Handle specific error codes
        if e.code == "rate_limit_exceeded":
            print("Please wait before trying again")
        elif e.code == "invalid_verification_code":
            print("The code you entered is incorrect")
        elif e.code == "max_attempts_exceeded":
            print("Too many failed attempts. Request a new code")
```

### Swift (iOS) Example

```swift
import Foundation

class RenovEasyAPI {
    private let baseURL = "https://api.renoveasy.com"
    private var accessToken: String?
    private var refreshToken: String?
    
    enum APIError: Error {
        case invalidResponse
        case serverError(code: String, message: String)
        case networkError(Error)
    }
    
    struct SendCodeRequest: Codable {
        let phone: String
        let countryCode: String
        
        enum CodingKeys: String, CodingKey {
            case phone
            case countryCode = "country_code"
        }
    }
    
    struct VerifyCodeRequest: Codable {
        let phone: String
        let countryCode: String
        let code: String
        
        enum CodingKeys: String, CodingKey {
            case phone
            case countryCode = "country_code"
            case code
        }
    }
    
    struct AuthResponse: Codable {
        let accessToken: String
        let refreshToken: String
        let expiresIn: Int
        let userType: String?
        let requiresTypeSelection: Bool
        
        enum CodingKeys: String, CodingKey {
            case accessToken = "access_token"
            case refreshToken = "refresh_token"
            case expiresIn = "expires_in"
            case userType = "user_type"
            case requiresTypeSelection = "requires_type_selection"
        }
    }
    
    func sendVerificationCode(phone: String, countryCode: String) async throws {
        let url = URL(string: "\(baseURL)/api/v1/auth/send-code")!
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.setValue("en", forHTTPHeaderField: "Accept-Language")
        
        let requestBody = SendCodeRequest(phone: phone, countryCode: countryCode)
        request.httpBody = try JSONEncoder().encode(requestBody)
        
        let (data, response) = try await URLSession.shared.data(for: request)
        
        guard let httpResponse = response as? HTTPURLResponse else {
            throw APIError.invalidResponse
        }
        
        if httpResponse.statusCode != 200 {
            // Handle error response
            if let errorData = try? JSONDecoder().decode(ErrorResponse.self, from: data) {
                throw APIError.serverError(
                    code: errorData.error.code,
                    message: errorData.error.message
                )
            }
            throw APIError.invalidResponse
        }
    }
    
    func verifyCode(phone: String, countryCode: String, code: String) async throws -> AuthResponse {
        let url = URL(string: "\(baseURL)/api/v1/auth/verify-code")!
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.setValue("en", forHTTPHeaderField: "Accept-Language")
        
        let requestBody = VerifyCodeRequest(
            phone: phone,
            countryCode: countryCode,
            code: code
        )
        request.httpBody = try JSONEncoder().encode(requestBody)
        
        let (data, response) = try await URLSession.shared.data(for: request)
        
        guard let httpResponse = response as? HTTPURLResponse else {
            throw APIError.invalidResponse
        }
        
        if httpResponse.statusCode != 200 {
            // Handle error response
            if let errorData = try? JSONDecoder().decode(ErrorResponse.self, from: data) {
                throw APIError.serverError(
                    code: errorData.error.code,
                    message: errorData.error.message
                )
            }
            throw APIError.invalidResponse
        }
        
        let authResponse = try JSONDecoder().decode(AuthResponse.self, from: data)
        
        // Store tokens
        self.accessToken = authResponse.accessToken
        self.refreshToken = authResponse.refreshToken
        
        // Store in Keychain for security (example using UserDefaults for simplicity)
        UserDefaults.standard.set(authResponse.accessToken, forKey: "access_token")
        UserDefaults.standard.set(authResponse.refreshToken, forKey: "refresh_token")
        
        return authResponse
    }
}

// Usage
Task {
    let api = RenovEasyAPI()
    
    do {
        // Send code
        try await api.sendVerificationCode(
            phone: "1234567890",
            countryCode: "+1"
        )
        print("Code sent successfully")
        
        // Verify code
        let authResult = try await api.verifyCode(
            phone: "1234567890",
            countryCode: "+1",
            code: "123456"
        )
        
        if authResult.requiresTypeSelection {
            print("User needs to select type")
            // Navigate to type selection screen
        } else {
            print("User authenticated as: \(authResult.userType ?? "unknown")")
            // Navigate to main app
        }
        
    } catch RenovEasyAPI.APIError.serverError(let code, let message) {
        print("Server error: \(code) - \(message)")
        
        switch code {
        case "rate_limit_exceeded":
            // Show rate limit message
            break
        case "invalid_verification_code":
            // Show invalid code message
            break
        default:
            // Show generic error
            break
        }
    } catch {
        print("Error: \(error)")
    }
}
```

### Kotlin (Android) Example

```kotlin
import retrofit2.Retrofit
import retrofit2.converter.gson.GsonConverterFactory
import retrofit2.http.*
import kotlinx.coroutines.*

// Data Classes
data class SendCodeRequest(
    val phone: String,
    @SerializedName("country_code") val countryCode: String
)

data class VerifyCodeRequest(
    val phone: String,
    @SerializedName("country_code") val countryCode: String,
    val code: String
)

data class AuthResponse(
    @SerializedName("access_token") val accessToken: String,
    @SerializedName("refresh_token") val refreshToken: String,
    @SerializedName("expires_in") val expiresIn: Int,
    @SerializedName("user_type") val userType: String?,
    @SerializedName("requires_type_selection") val requiresTypeSelection: Boolean
)

data class ApiError(
    val code: String,
    val message: String
)

data class ErrorResponse(
    val error: ApiError
)

// Retrofit Interface
interface RenovEasyApi {
    @POST("api/v1/auth/send-code")
    suspend fun sendCode(
        @Body request: SendCodeRequest,
        @Header("Accept-Language") language: String = "en"
    ): Response<SendCodeResponse>
    
    @POST("api/v1/auth/verify-code")
    suspend fun verifyCode(
        @Body request: VerifyCodeRequest,
        @Header("Accept-Language") language: String = "en"
    ): Response<AuthResponse>
    
    @POST("api/v1/auth/select-type")
    suspend fun selectType(
        @Body request: SelectTypeRequest,
        @Header("Authorization") token: String,
        @Header("Accept-Language") language: String = "en"
    ): Response<AuthResponse>
    
    @POST("api/v1/auth/refresh")
    suspend fun refreshToken(
        @Body request: RefreshTokenRequest,
        @Header("Accept-Language") language: String = "en"
    ): Response<AuthResponse>
    
    @POST("api/v1/auth/logout")
    suspend fun logout(
        @Header("Authorization") token: String,
        @Header("Accept-Language") language: String = "en"
    ): Response<LogoutResponse>
}

// API Client
class RenovEasyClient(private val baseUrl: String = "https://api.renoveasy.com") {
    private val api: RenovEasyApi
    private var accessToken: String? = null
    private var refreshToken: String? = null
    
    init {
        val retrofit = Retrofit.Builder()
            .baseUrl(baseUrl)
            .addConverterFactory(GsonConverterFactory.create())
            .build()
        
        api = retrofit.create(RenovEasyApi::class.java)
    }
    
    suspend fun sendVerificationCode(phone: String, countryCode: String): Result<SendCodeResponse> {
        return try {
            val response = api.sendCode(SendCodeRequest(phone, countryCode))
            if (response.isSuccessful) {
                Result.success(response.body()!!)
            } else {
                val error = parseError(response)
                Result.failure(ApiException(error.code, error.message))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
    
    suspend fun verifyCode(
        phone: String, 
        countryCode: String, 
        code: String
    ): Result<AuthResponse> {
        return try {
            val response = api.verifyCode(
                VerifyCodeRequest(phone, countryCode, code)
            )
            
            if (response.isSuccessful) {
                val authResponse = response.body()!!
                
                // Store tokens
                accessToken = authResponse.accessToken
                refreshToken = authResponse.refreshToken
                
                // Save to SharedPreferences or secure storage
                saveTokens(authResponse.accessToken, authResponse.refreshToken)
                
                Result.success(authResponse)
            } else {
                val error = parseError(response)
                Result.failure(ApiException(error.code, error.message))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
    
    private fun parseError(response: Response<*>): ApiError {
        return try {
            val errorBody = response.errorBody()?.string()
            val gson = Gson()
            val errorResponse = gson.fromJson(errorBody, ErrorResponse::class.java)
            errorResponse.error
        } catch (e: Exception) {
            ApiError("unknown_error", "An unknown error occurred")
        }
    }
    
    private fun saveTokens(accessToken: String, refreshToken: String) {
        // Use Android Keystore or EncryptedSharedPreferences for security
        val sharedPref = context.getSharedPreferences("auth", Context.MODE_PRIVATE)
        with(sharedPref.edit()) {
            putString("access_token", accessToken)
            putString("refresh_token", refreshToken)
            apply()
        }
    }
}

class ApiException(val code: String, message: String) : Exception(message)

// Usage in Activity/Fragment
class LoginActivity : AppCompatActivity() {
    private val client = RenovEasyClient()
    
    fun sendCode(phone: String, countryCode: String) {
        lifecycleScope.launch {
            when (val result = client.sendVerificationCode(phone, countryCode)) {
                is Result.Success -> {
                    // Show success message
                    showMessage("Code sent successfully")
                }
                is Result.Failure -> {
                    val error = result.exception as? ApiException
                    when (error?.code) {
                        "rate_limit_exceeded" -> {
                            showMessage("Too many requests. Please wait.")
                        }
                        "invalid_phone_format" -> {
                            showMessage("Invalid phone number")
                        }
                        else -> {
                            showMessage(error?.message ?: "Error sending code")
                        }
                    }
                }
            }
        }
    }
}
```

## API Versioning

The API uses URL-based versioning. The current version is `v1`.

### Version Format
```
/api/v{major_version}/
```

### Deprecation Policy
- New versions will be announced at least 3 months before deprecating old versions
- Deprecated endpoints will return a `Deprecation` header with the sunset date
- Documentation will clearly mark deprecated features

### Migration Guide
When migrating to a new API version:
1. Review the changelog for breaking changes
2. Update your base URL to the new version
3. Test thoroughly in a development environment
4. Update error handling for any new error codes
5. Gradually migrate production traffic

## Support

For API support and questions:
- Email: api-support@renoveasy.com
- Documentation: https://docs.renoveasy.com/api
- Status Page: https://status.renoveasy.com

## Changelog

### Version 1.0.0 (Current)
- Initial release
- Phone-based authentication
- SMS verification
- JWT token management
- User type selection
- Bilingual support (Chinese/English)
- Comprehensive rate limiting
- Security headers and CORS support

---

Last Updated: January 19, 2025