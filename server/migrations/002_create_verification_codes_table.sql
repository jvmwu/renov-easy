-- Migration: 002_create_verification_codes_table
-- Description: Create verification_codes table for OTP management
-- Date: 2025-08-21
-- Requirements: Auth-Passwordless Specification - Requirements 2.1, 4.1, 8.1

-- ============================================================================
-- UP MIGRATION
-- ============================================================================

-- Create verification_codes table for storing OTP codes
CREATE TABLE IF NOT EXISTS verification_codes (
    -- Primary key using UUID
    id CHAR(36) NOT NULL,
    
    -- Phone number hash for security (SHA-256)
    phone_hash VARCHAR(64) NOT NULL,
    
    -- Encrypted verification code (AES-256-GCM)
    code_encrypted VARCHAR(255) NOT NULL,
    
    -- Attempt tracking (max 3 attempts as per requirement)
    attempts TINYINT UNSIGNED NOT NULL DEFAULT 0,
    max_attempts TINYINT UNSIGNED NOT NULL DEFAULT 3,
    
    -- Lifecycle management (5-minute expiry as per requirement 2.1)
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NOT NULL,
    
    -- Status flags
    is_used BOOLEAN NOT NULL DEFAULT FALSE,
    is_locked BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- Request metadata for audit
    ip_address VARCHAR(45) NULL,
    user_agent TEXT NULL,
    
    -- Constraints
    PRIMARY KEY (id),
    INDEX idx_verification_codes_phone_hash (phone_hash),
    INDEX idx_verification_codes_expires_at (expires_at),
    INDEX idx_verification_codes_is_used (is_used),
    INDEX idx_verification_codes_created_at (created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Add table comment for documentation
ALTER TABLE verification_codes COMMENT = 'Stores OTP verification codes for passwordless authentication with 5-minute TTL';

-- Add column comments for verification_codes
ALTER TABLE verification_codes
    MODIFY COLUMN id CHAR(36) NOT NULL COMMENT 'Unique identifier for verification code (UUID v4)',
    MODIFY COLUMN phone_hash VARCHAR(64) NOT NULL COMMENT 'SHA-256 hash of the phone number for privacy',
    MODIFY COLUMN code_encrypted VARCHAR(255) NOT NULL COMMENT 'AES-256-GCM encrypted 6-digit verification code',
    MODIFY COLUMN attempts TINYINT UNSIGNED NOT NULL DEFAULT 0 COMMENT 'Number of verification attempts made',
    MODIFY COLUMN max_attempts TINYINT UNSIGNED NOT NULL DEFAULT 3 COMMENT 'Maximum allowed verification attempts',
    MODIFY COLUMN created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Code creation timestamp',
    MODIFY COLUMN expires_at TIMESTAMP NOT NULL COMMENT 'Code expiration timestamp (5 minutes from creation)',
    MODIFY COLUMN is_used BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether code has been successfully used',
    MODIFY COLUMN is_locked BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether code is locked due to max attempts',
    MODIFY COLUMN ip_address VARCHAR(45) NULL COMMENT 'IP address from which code was requested',
    MODIFY COLUMN user_agent TEXT NULL COMMENT 'User agent of the requesting client';

-- ============================================================================
-- DOWN MIGRATION
-- ============================================================================
-- To rollback this migration, uncomment and run:
-- DROP TABLE IF EXISTS verification_codes;