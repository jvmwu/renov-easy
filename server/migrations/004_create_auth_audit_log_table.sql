-- Migration: 004_create_auth_audit_log_table
-- Description: Create auth_audit_log table for security monitoring and compliance
-- Date: 2025-08-21

-- Create auth_audit_log table for security monitoring
CREATE TABLE IF NOT EXISTS auth_audit_log (
    -- Primary key using UUID
    id CHAR(36) NOT NULL,
    
    -- User reference (nullable for failed attempts before user creation)
    user_id CHAR(36) NULL,
    
    -- Phone hash for tracking attempts even without user_id
    phone_hash VARCHAR(64) NULL,
    
    -- Audit information
    action VARCHAR(50) NOT NULL,
    success BOOLEAN NOT NULL,
    
    -- Request metadata
    ip_address VARCHAR(45) NULL,
    user_agent TEXT NULL,
    
    -- Error tracking
    error_code VARCHAR(50) NULL,
    error_message TEXT NULL,
    
    -- Additional context (JSON format for flexibility)
    metadata JSON NULL,
    
    -- Timestamp
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Constraints
    PRIMARY KEY (id),
    CONSTRAINT fk_auth_audit_log_user_id 
        FOREIGN KEY (user_id) REFERENCES users(id) 
        ON DELETE SET NULL
        ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Create indexes for auth_audit_log table
CREATE INDEX idx_auth_audit_log_user_id ON auth_audit_log(user_id);
CREATE INDEX idx_auth_audit_log_phone_hash ON auth_audit_log(phone_hash);
CREATE INDEX idx_auth_audit_log_action ON auth_audit_log(action);
CREATE INDEX idx_auth_audit_log_created_at ON auth_audit_log(created_at);
CREATE INDEX idx_auth_audit_log_success ON auth_audit_log(success);
CREATE INDEX idx_auth_audit_log_ip_address ON auth_audit_log(ip_address);

-- Add table comment for documentation
ALTER TABLE auth_audit_log COMMENT = 'Audit log for authentication events and security monitoring';

-- Add column comments for auth_audit_log
ALTER TABLE auth_audit_log
    MODIFY COLUMN id CHAR(36) NOT NULL COMMENT 'Unique identifier for audit entry (UUID v4)',
    MODIFY COLUMN user_id CHAR(36) NULL COMMENT 'User ID if authentication was successful or user exists',
    MODIFY COLUMN phone_hash VARCHAR(64) NULL COMMENT 'Phone hash for tracking attempts without user_id',
    MODIFY COLUMN action VARCHAR(50) NOT NULL COMMENT 'Action type: send_code, verify_code, refresh_token, logout, etc.',
    MODIFY COLUMN success BOOLEAN NOT NULL COMMENT 'Whether the action was successful',
    MODIFY COLUMN ip_address VARCHAR(45) NULL COMMENT 'Client IP address (supports IPv4 and IPv6)',
    MODIFY COLUMN user_agent TEXT NULL COMMENT 'HTTP User-Agent header',
    MODIFY COLUMN error_code VARCHAR(50) NULL COMMENT 'Error code if action failed',
    MODIFY COLUMN error_message TEXT NULL COMMENT 'Detailed error message for debugging',
    MODIFY COLUMN metadata JSON NULL COMMENT 'Additional context in JSON format',
    MODIFY COLUMN created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Audit entry timestamp';