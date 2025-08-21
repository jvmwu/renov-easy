-- Migration: 003_create_refresh_tokens_table
-- Description: Create refresh_tokens table for JWT refresh token management
-- Date: 2025-08-21

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
    
    -- Device/client information (optional, for future use)
    device_info VARCHAR(255) NULL,
    ip_address VARCHAR(45) NULL,
    
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

-- Add table comment for documentation
ALTER TABLE refresh_tokens COMMENT = 'Stores refresh tokens for JWT authentication';

-- Add column comments for refresh_tokens
ALTER TABLE refresh_tokens
    MODIFY COLUMN id CHAR(36) NOT NULL COMMENT 'Unique identifier for refresh token (UUID v4)',
    MODIFY COLUMN user_id CHAR(36) NOT NULL COMMENT 'Reference to user who owns this token',
    MODIFY COLUMN token_hash VARCHAR(64) NOT NULL COMMENT 'SHA-256 hash of the refresh token',
    MODIFY COLUMN created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Token creation timestamp',
    MODIFY COLUMN expires_at TIMESTAMP NOT NULL COMMENT 'Token expiration timestamp (30 days from creation)',
    MODIFY COLUMN last_used_at TIMESTAMP NULL COMMENT 'Last time this token was used to refresh access token',
    MODIFY COLUMN is_revoked BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether token has been manually revoked',
    MODIFY COLUMN device_info VARCHAR(255) NULL COMMENT 'Optional device/client identification',
    MODIFY COLUMN ip_address VARCHAR(45) NULL COMMENT 'IP address from which token was created';