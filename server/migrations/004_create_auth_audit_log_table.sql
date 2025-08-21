-- Migration: 004_create_auth_audit_log_table
-- Description: Create auth_audit_log table for security monitoring and compliance
-- Date: 2025-08-21
-- Requirements: Auth-Passwordless Specification - Requirements 7.1, 7.8

-- ============================================================================
-- UP MIGRATION
-- ============================================================================

-- Create auth_audit_log table for security monitoring and compliance (Requirement 7.1)
CREATE TABLE IF NOT EXISTS auth_audit_log (
    -- Primary key using UUID
    id CHAR(36) NOT NULL,
    
    -- Event classification (Requirement 7.1: log all authentication events)
    event_type ENUM(
        'login_attempt',
        'login_success',
        'login_failed',
        'send_code_request',
        'send_code_success',
        'send_code_failed',
        'verify_code_attempt',
        'verify_code_success',
        'verify_code_failed',
        'token_generated',
        'token_refreshed',
        'token_revoked',
        'rate_limit_exceeded',
        'account_locked',
        'account_unlocked',
        'logout'
    ) NOT NULL,
    
    -- User reference (nullable for failed attempts before user creation)
    user_id CHAR(36) NULL,
    
    -- Phone tracking with masking (Requirement 7.8: phone masking)
    phone_masked VARCHAR(20) NULL COMMENT 'Masked phone number showing only last 4 digits',
    phone_hash VARCHAR(64) NULL COMMENT 'SHA-256 hash for correlation without exposing phone',
    
    -- Request metadata (Requirement 7.2, 7.3: IP and device info)
    ip_address VARCHAR(45) NOT NULL,
    user_agent TEXT NULL,
    device_fingerprint VARCHAR(255) NULL,
    
    -- Action details
    action VARCHAR(100) NOT NULL,
    success BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- Error tracking (Requirement 7.2: failure reasons)
    error_code VARCHAR(50) NULL,
    error_message TEXT NULL,
    
    -- Token tracking (Requirement 7.5: token generation logging)
    token_id CHAR(36) NULL,
    token_type ENUM('access', 'refresh') NULL,
    
    -- Rate limiting info (Requirement 7.4: rate limit violations)
    rate_limit_type VARCHAR(50) NULL,
    rate_limit_remaining INT NULL,
    
    -- Additional context (JSON format for flexibility)
    metadata JSON NULL,
    
    -- Timestamps
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Archival support (Requirement 7.6: 90-day retention)
    archived BOOLEAN NOT NULL DEFAULT FALSE,
    archived_at TIMESTAMP NULL,
    
    -- Constraints
    PRIMARY KEY (id),
    CONSTRAINT fk_auth_audit_log_user_id 
        FOREIGN KEY (user_id) REFERENCES users(id) 
        ON DELETE SET NULL
        ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Create indexes for auth_audit_log table (optimized for query patterns)
CREATE INDEX idx_auth_audit_log_user_id ON auth_audit_log(user_id);
CREATE INDEX idx_auth_audit_log_phone_hash ON auth_audit_log(phone_hash);
CREATE INDEX idx_auth_audit_log_event_type ON auth_audit_log(event_type);
CREATE INDEX idx_auth_audit_log_created_at ON auth_audit_log(created_at);
CREATE INDEX idx_auth_audit_log_success ON auth_audit_log(success);
CREATE INDEX idx_auth_audit_log_ip_address ON auth_audit_log(ip_address);
CREATE INDEX idx_auth_audit_log_archived ON auth_audit_log(archived);
CREATE INDEX idx_auth_audit_log_token_id ON auth_audit_log(token_id);

-- Composite indexes for common query patterns
CREATE INDEX idx_auth_audit_log_user_event ON auth_audit_log(user_id, event_type, created_at);
CREATE INDEX idx_auth_audit_log_phone_event ON auth_audit_log(phone_hash, event_type, created_at);
CREATE INDEX idx_auth_audit_log_archival ON auth_audit_log(archived, created_at);

-- Add table comment for documentation
ALTER TABLE auth_audit_log COMMENT = 'Immutable audit log for authentication events, security monitoring, and compliance (Requirements 7.1, 7.8)';

-- Add column comments for auth_audit_log
ALTER TABLE auth_audit_log
    MODIFY COLUMN id CHAR(36) NOT NULL COMMENT 'Unique identifier for audit entry (UUID v4)',
    MODIFY COLUMN event_type ENUM('login_attempt','login_success','login_failed','send_code_request','send_code_success','send_code_failed','verify_code_attempt','verify_code_success','verify_code_failed','token_generated','token_refreshed','token_revoked','rate_limit_exceeded','account_locked','account_unlocked','logout') NOT NULL COMMENT 'Type of authentication event (Requirement 7.1)',
    MODIFY COLUMN user_id CHAR(36) NULL COMMENT 'User ID if known (null for pre-auth events)',
    MODIFY COLUMN phone_masked VARCHAR(20) NULL COMMENT 'Masked phone showing only last 4 digits (Requirement 7.8)',
    MODIFY COLUMN phone_hash VARCHAR(64) NULL COMMENT 'SHA-256 hash for phone correlation without exposure',
    MODIFY COLUMN ip_address VARCHAR(45) NOT NULL COMMENT 'Client IP address (IPv4/IPv6) for security tracking',
    MODIFY COLUMN user_agent TEXT NULL COMMENT 'HTTP User-Agent for device identification',
    MODIFY COLUMN device_fingerprint VARCHAR(255) NULL COMMENT 'Device fingerprint for multi-device tracking',
    MODIFY COLUMN action VARCHAR(100) NOT NULL COMMENT 'Detailed action description',
    MODIFY COLUMN success BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether the action succeeded',
    MODIFY COLUMN error_code VARCHAR(50) NULL COMMENT 'Standardized error code for failed actions',
    MODIFY COLUMN error_message TEXT NULL COMMENT 'Detailed error message for debugging',
    MODIFY COLUMN token_id CHAR(36) NULL COMMENT 'Token ID for token-related events (Requirement 7.5)',
    MODIFY COLUMN token_type ENUM('access', 'refresh') NULL COMMENT 'Type of token for token events',
    MODIFY COLUMN rate_limit_type VARCHAR(50) NULL COMMENT 'Type of rate limit violated (Requirement 7.4)',
    MODIFY COLUMN rate_limit_remaining INT NULL COMMENT 'Remaining attempts before rate limit reset',
    MODIFY COLUMN metadata JSON NULL COMMENT 'Additional context in JSON format',
    MODIFY COLUMN created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Immutable timestamp of event',
    MODIFY COLUMN archived BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether record has been archived (Requirement 7.6)',
    MODIFY COLUMN archived_at TIMESTAMP NULL COMMENT 'Timestamp when record was archived';

-- Create stored procedure for archiving old audit logs (90-day retention per Requirement 7.6)
DELIMITER $$

CREATE PROCEDURE IF NOT EXISTS archive_old_audit_logs()
BEGIN
    -- Mark logs older than 90 days as archived
    UPDATE auth_audit_log 
    SET archived = TRUE,
        archived_at = NOW()
    WHERE created_at < DATE_SUB(NOW(), INTERVAL 90 DAY)
    AND archived = FALSE;
    
    -- Optional: Move archived records to separate table or export to cold storage
    -- This can be implemented based on specific infrastructure requirements
END$$

DELIMITER ;

-- Create event scheduler to run archival daily (optional, requires EVENT privilege)
-- CREATE EVENT IF NOT EXISTS archive_audit_logs_daily
-- ON SCHEDULE EVERY 1 DAY
-- STARTS (DATE(NOW()) + INTERVAL 1 DAY)
-- DO CALL archive_old_audit_logs();

-- ============================================================================
-- DOWN MIGRATION
-- ============================================================================
-- To rollback this migration, uncomment and run:
-- DROP PROCEDURE IF EXISTS archive_old_audit_logs;
-- DROP TABLE IF EXISTS auth_audit_log;