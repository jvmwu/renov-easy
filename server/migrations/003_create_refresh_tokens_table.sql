-- Migration: 003_create_refresh_tokens_table
-- Description: Create refresh_tokens table for JWT refresh token management
-- Date: 2025-08-21
-- Requirements: Auth-Passwordless Specification - Requirements 5.2, 5.5

-- ============================================================================
-- UP MIGRATION
-- ============================================================================

-- Create refresh_tokens table for JWT refresh token management
CREATE TABLE IF NOT EXISTS refresh_tokens (
    -- Primary key using UUID
    id CHAR(36) NOT NULL,
    
    -- Foreign key to users table
    user_id CHAR(36) NOT NULL,
    
    -- Token hash for security (store hash instead of plaintext)
    token_hash VARCHAR(64) NOT NULL,
    
    -- Token lifecycle management
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NOT NULL,
    last_used_at TIMESTAMP NULL,
    
    -- Token status
    is_revoked BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- Device/client information for multi-device support
    device_id VARCHAR(255) NULL,
    device_name VARCHAR(255) NULL,
    device_type ENUM('ios', 'android', 'web', 'harmony', 'unknown') NULL DEFAULT 'unknown',
    ip_address VARCHAR(45) NULL,
    user_agent TEXT NULL,
    
    -- Token rotation tracking (Requirement 5.5: rotation on use)
    rotated_from_token_id CHAR(36) NULL,
    rotated_to_token_id CHAR(36) NULL,
    
    -- Enhanced revocation tracking
    revoked_at TIMESTAMP NULL,
    revoke_reason VARCHAR(255) NULL,
    
    -- Usage tracking
    use_count INT UNSIGNED NOT NULL DEFAULT 0,
    
    -- Update tracking
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    -- Constraints
    PRIMARY KEY (id),
    UNIQUE KEY uk_token_hash (token_hash),
    CONSTRAINT fk_refresh_tokens_user_id 
        FOREIGN KEY (user_id) REFERENCES users(id) 
        ON DELETE CASCADE
        ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Create indexes for refresh_tokens table
CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_tokens_expires_at ON refresh_tokens(expires_at);
CREATE INDEX idx_refresh_tokens_is_revoked ON refresh_tokens(is_revoked);
CREATE INDEX idx_refresh_tokens_last_used_at ON refresh_tokens(last_used_at);
CREATE INDEX idx_refresh_tokens_device_id ON refresh_tokens(device_id);
CREATE INDEX idx_refresh_tokens_rotated_from ON refresh_tokens(rotated_from_token_id);
CREATE INDEX idx_refresh_tokens_created_at ON refresh_tokens(created_at);

-- Add table comment for documentation
ALTER TABLE refresh_tokens COMMENT = 'Stores JWT refresh tokens for session management with 30-day expiry and rotation support (Requirements 5.2, 5.5)';

-- Add column comments for refresh_tokens
ALTER TABLE refresh_tokens
    MODIFY COLUMN id CHAR(36) NOT NULL COMMENT 'Unique identifier for refresh token (UUID v4)',
    MODIFY COLUMN user_id CHAR(36) NOT NULL COMMENT 'Reference to the user who owns this token',
    MODIFY COLUMN token_hash VARCHAR(64) NOT NULL COMMENT 'SHA-256 hash of the actual refresh token for security (Requirement 5.5)',
    MODIFY COLUMN created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Token creation timestamp',
    MODIFY COLUMN expires_at TIMESTAMP NOT NULL COMMENT 'Token expiration timestamp (30 days from creation per Requirement 5.2)',
    MODIFY COLUMN last_used_at TIMESTAMP NULL COMMENT 'Most recent usage timestamp for activity tracking',
    MODIFY COLUMN is_revoked BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether token has been manually revoked',
    MODIFY COLUMN device_id VARCHAR(255) NULL COMMENT 'Unique device identifier for multi-device tracking',
    MODIFY COLUMN device_name VARCHAR(255) NULL COMMENT 'Human-readable device name (e.g., "John iPhone 15")',
    MODIFY COLUMN device_type ENUM('ios', 'android', 'web', 'harmony', 'unknown') NULL DEFAULT 'unknown' COMMENT 'Platform type of the client device',
    MODIFY COLUMN ip_address VARCHAR(45) NULL COMMENT 'IP address from which token was issued (supports IPv6)',
    MODIFY COLUMN user_agent TEXT NULL COMMENT 'User agent string of the client',
    MODIFY COLUMN rotated_from_token_id CHAR(36) NULL COMMENT 'Previous token ID if this token was created via rotation',
    MODIFY COLUMN rotated_to_token_id CHAR(36) NULL COMMENT 'New token ID if this token was rotated (Requirement 5.5)',
    MODIFY COLUMN revoked_at TIMESTAMP NULL COMMENT 'Timestamp when token was revoked',
    MODIFY COLUMN revoke_reason VARCHAR(255) NULL COMMENT 'Reason for token revocation (e.g., "logout", "security", "password_change")',
    MODIFY COLUMN use_count INT UNSIGNED NOT NULL DEFAULT 0 COMMENT 'Number of times token has been used',
    MODIFY COLUMN updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Last modification timestamp';

-- Create stored procedure for token cleanup (optional, for maintenance)
DELIMITER $$

CREATE PROCEDURE IF NOT EXISTS cleanup_expired_refresh_tokens()
BEGIN
    -- Delete expired tokens older than 7 days (grace period after expiry)
    DELETE FROM refresh_tokens 
    WHERE expires_at < DATE_SUB(NOW(), INTERVAL 7 DAY);
    
    -- Delete revoked tokens older than 30 days
    DELETE FROM refresh_tokens 
    WHERE is_revoked = TRUE 
    AND revoked_at < DATE_SUB(NOW(), INTERVAL 30 DAY);
END$$

DELIMITER ;

-- ============================================================================
-- DOWN MIGRATION
-- ============================================================================
-- To rollback this migration, uncomment and run:
-- DROP PROCEDURE IF EXISTS cleanup_expired_refresh_tokens;
-- DROP TABLE IF EXISTS refresh_tokens;