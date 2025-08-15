-- Migration: 001_create_users_table
-- Description: Create users table for RenovEasy authentication system
-- Date: 2025-08-15

-- Create users table
CREATE TABLE IF NOT EXISTS users (
    -- Primary key using UUID (stored as CHAR(36))
    id CHAR(36) NOT NULL,
    
    -- Phone number stored as hash for privacy (SHA-256 produces 64 character hex string)
    phone_hash VARCHAR(64) NOT NULL,
    
    -- Country code for international phone numbers (e.g., +86, +61)
    country_code VARCHAR(10) NOT NULL,
    
    -- User type: either 'customer' or 'worker', nullable until user selects
    user_type ENUM('customer', 'worker') NULL,
    
    -- Timestamps for audit trail
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    last_login_at TIMESTAMP NULL,
    
    -- Account status flags
    is_verified BOOLEAN NOT NULL DEFAULT FALSE,
    is_blocked BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- Constraints
    PRIMARY KEY (id),
    UNIQUE KEY uk_phone_hash (phone_hash)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Create indexes for performance optimization
CREATE INDEX idx_user_type ON users(user_type);
CREATE INDEX idx_created_at ON users(created_at);
CREATE INDEX idx_is_blocked ON users(is_blocked);

-- Add comments for documentation
ALTER TABLE users COMMENT = 'Stores user authentication and profile information';

-- Column comments
ALTER TABLE users
    MODIFY COLUMN id CHAR(36) NOT NULL COMMENT 'Unique identifier for user (UUID v4)',
    MODIFY COLUMN phone_hash VARCHAR(64) NOT NULL COMMENT 'SHA-256 hash of phone number for privacy',
    MODIFY COLUMN country_code VARCHAR(10) NOT NULL COMMENT 'International dialing code (e.g., +86 for China, +61 for Australia)',
    MODIFY COLUMN user_type ENUM('customer', 'worker') NULL COMMENT 'User role in the platform',
    MODIFY COLUMN created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Account creation timestamp',
    MODIFY COLUMN updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Last modification timestamp',
    MODIFY COLUMN last_login_at TIMESTAMP NULL COMMENT 'Most recent successful login timestamp',
    MODIFY COLUMN is_verified BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether phone number has been verified',
    MODIFY COLUMN is_blocked BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether account is blocked for security reasons';