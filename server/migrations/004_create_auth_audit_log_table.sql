-- Migration: 004_create_auth_audit_log_table
-- Description: Create auth_audit_log table for security monitoring and compliance with enhanced fields
-- Date: 2025-08-21 (Enhanced: 2025-08-24)
-- Requirements: Auth-Passwordless Specification - Requirements 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7, 7.8

-- ============================================================================
-- UP MIGRATION
-- ============================================================================

-- Create auth_audit_log table for security monitoring and compliance (Requirement 7.1, 7.7)
CREATE TABLE IF NOT EXISTS auth_audit_log (
    -- Primary key using UUID
    id CHAR(36) NOT NULL,
    
    -- Event classification (Requirement 7.1, 7.7: enhanced event tracking)
    event_type VARCHAR(50) NOT NULL,
    
    -- User reference (nullable for failed attempts before user creation)
    user_id CHAR(36) NULL,
    
    -- Phone tracking with masking (Requirement 7.8: phone masking)
    phone_masked VARCHAR(20) NULL COMMENT 'Masked phone number showing only last 4 digits',
    phone_hash VARCHAR(64) NULL COMMENT 'SHA-256 hash for correlation without exposing phone',
    
    -- Request metadata (Requirement 7.2, 7.3: IP and device info)
    ip_address VARCHAR(45) NOT NULL,
    user_agent TEXT NULL,
    device_fingerprint VARCHAR(255) NULL,
    device_info VARCHAR(255) NULL COMMENT 'Extracted device information from user agent',
    
    -- Action details
    action VARCHAR(100) NOT NULL,
    success BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- Error tracking (Requirement 7.2: failure reasons)
    error_code VARCHAR(50) NULL,
    error_message TEXT NULL,
    failure_reason VARCHAR(500) NULL COMMENT 'Detailed failure reason for failed attempts',
    
    -- Token tracking (Requirement 7.5: token generation logging)
    token_id CHAR(36) NULL,
    token_type ENUM('access', 'refresh') NULL,
    
    -- Rate limiting info (Requirement 7.4: rate limit violations)
    rate_limit_type VARCHAR(50) NULL,
    rate_limit_remaining INT NULL,
    
    -- Additional context (JSON format for flexibility)
    metadata JSON NULL,
    event_data JSON NULL COMMENT 'Additional event context in JSON format',
    
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
CREATE INDEX idx_auth_audit_log_rate_limit_type ON auth_audit_log(rate_limit_type);

-- Composite indexes for common query patterns
CREATE INDEX idx_auth_audit_log_user_event ON auth_audit_log(user_id, event_type, created_at);
CREATE INDEX idx_auth_audit_log_phone_event ON auth_audit_log(phone_hash, event_type, created_at);
CREATE INDEX idx_auth_audit_log_archival ON auth_audit_log(archived, created_at);
CREATE INDEX idx_auth_audit_log_event_tracking ON auth_audit_log(event_type, created_at, success);

-- Add table comment for documentation
ALTER TABLE auth_audit_log COMMENT = 'Immutable audit log for authentication events, security monitoring, and compliance (Requirements 7.1-7.8)';

-- Add column comments for auth_audit_log
ALTER TABLE auth_audit_log
    MODIFY COLUMN id CHAR(36) NOT NULL COMMENT 'Unique identifier for audit entry (UUID v4)',
    MODIFY COLUMN event_type VARCHAR(50) NOT NULL COMMENT 'Type of authentication event (Requirement 7.1, 7.7)',
    MODIFY COLUMN user_id CHAR(36) NULL COMMENT 'User ID if known (null for pre-auth events)',
    MODIFY COLUMN phone_masked VARCHAR(20) NULL COMMENT 'Masked phone showing only last 4 digits (e.g., ****1234) (Requirement 7.8)',
    MODIFY COLUMN phone_hash VARCHAR(64) NULL COMMENT 'SHA-256 hash for phone correlation without exposure',
    MODIFY COLUMN ip_address VARCHAR(45) NOT NULL COMMENT 'Client IP address (IPv4/IPv6) for security tracking',
    MODIFY COLUMN user_agent TEXT NULL COMMENT 'HTTP User-Agent for device identification',
    MODIFY COLUMN device_fingerprint VARCHAR(255) NULL COMMENT 'Device fingerprint for multi-device tracking',
    MODIFY COLUMN device_info VARCHAR(255) NULL COMMENT 'Device type and OS extracted from user agent (e.g., Mobile/iOS)',
    MODIFY COLUMN action VARCHAR(100) NOT NULL COMMENT 'Detailed action description',
    MODIFY COLUMN success BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether the action succeeded',
    MODIFY COLUMN error_code VARCHAR(50) NULL COMMENT 'Standardized error code for failed actions',
    MODIFY COLUMN error_message TEXT NULL COMMENT 'Detailed error message for debugging',
    MODIFY COLUMN failure_reason VARCHAR(500) NULL COMMENT 'Human-readable failure reason for debugging and analysis',
    MODIFY COLUMN token_id CHAR(36) NULL COMMENT 'Token ID for token-related events (Requirement 7.5)',
    MODIFY COLUMN token_type ENUM('access', 'refresh') NULL COMMENT 'Type of token for token events',
    MODIFY COLUMN rate_limit_type VARCHAR(50) NULL COMMENT 'Type of rate limit (phone/ip/global) (Requirement 7.4)',
    MODIFY COLUMN rate_limit_remaining INT NULL COMMENT 'Remaining attempts before rate limit reset',
    MODIFY COLUMN metadata JSON NULL COMMENT 'Additional context in JSON format',
    MODIFY COLUMN event_data JSON NULL COMMENT 'Additional structured event data for comprehensive tracking',
    MODIFY COLUMN created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Immutable timestamp of event',
    MODIFY COLUMN archived BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether record has been archived (Requirement 7.6)',
    MODIFY COLUMN archived_at TIMESTAMP NULL COMMENT 'Timestamp when record was archived';

-- Create stored procedure for archiving old audit logs (90-day retention per Requirement 7.6)
DELIMITER $$

CREATE PROCEDURE IF NOT EXISTS archive_old_audit_logs()
BEGIN
    DECLARE archived_count INT DEFAULT 0;
    
    -- Start transaction
    START TRANSACTION;
    
    -- Mark logs older than 90 days as archived
    UPDATE auth_audit_log 
    SET archived = TRUE,
        archived_at = NOW()
    WHERE created_at < DATE_SUB(NOW(), INTERVAL 90 DAY)
    AND archived = FALSE;
    
    -- Get count of archived records
    SET archived_count = ROW_COUNT();
    
    -- Log the archival operation in the audit log itself
    IF archived_count > 0 THEN
        INSERT INTO auth_audit_log (
            id,
            event_type,
            action,
            success,
            ip_address,
            event_data,
            created_at
        ) VALUES (
            UUID(),
            'ARCHIVAL_COMPLETED',
            'ARCHIVAL_COMPLETED',
            TRUE,
            '127.0.0.1',
            JSON_OBJECT('archived_count', archived_count, 'archival_date', NOW()),
            NOW()
        );
    END IF;
    
    COMMIT;
    
    -- Return the count of archived records
    SELECT archived_count AS records_archived;
END$$

DELIMITER ;

-- Create function to mask phone numbers consistently
DELIMITER $$

CREATE FUNCTION IF NOT EXISTS mask_phone_number(phone VARCHAR(50))
RETURNS VARCHAR(20)
DETERMINISTIC
BEGIN
    DECLARE digits VARCHAR(50);
    DECLARE masked VARCHAR(20);
    
    -- Extract only digits
    SET digits = REGEXP_REPLACE(phone, '[^0-9]', '');
    
    -- If less than or equal to 4 digits, mask all
    IF LENGTH(digits) <= 4 THEN
        SET masked = REPEAT('*', LENGTH(digits));
    ELSE
        -- Show only last 4 digits
        SET masked = CONCAT('****', RIGHT(digits, 4));
    END IF;
    
    RETURN masked;
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
-- DROP FUNCTION IF EXISTS mask_phone_number;
-- DROP PROCEDURE IF EXISTS archive_old_audit_logs;
-- DROP TABLE IF EXISTS auth_audit_log;