-- Migration: 002_create_verification_codes_and_enhance_tokens
-- Description: Create verification_codes table for OTP management and enhance refresh_tokens with rotation tracking
-- Date: 2025-08-21 (Updated: 2025-08-24)
-- Requirements: Auth-Passwordless Specification - Requirements 2.1, 4.1, 5.5, 8.1

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
-- ENHANCE REFRESH TOKENS STORAGE
-- ============================================================================
-- Requirements: Auth-Passwordless Specification - Requirement 5.5 (JWT Token Generation)

-- Add token_family column for token rotation tracking
ALTER TABLE refresh_tokens
    ADD COLUMN IF NOT EXISTS token_family VARCHAR(36) NULL COMMENT 'Token family ID for rotation tracking - all tokens in a rotation chain share this ID'
    AFTER rotated_to_token_id;

-- Add enhanced device fingerprint for better security
ALTER TABLE refresh_tokens
    ADD COLUMN IF NOT EXISTS device_fingerprint VARCHAR(255) NULL COMMENT 'Device fingerprint hash for enhanced security tracking'
    AFTER device_type;

-- Add previous_token_id for complete rotation chain tracking
ALTER TABLE refresh_tokens
    ADD COLUMN IF NOT EXISTS previous_token_id CHAR(36) NULL COMMENT 'Direct reference to the previous token in rotation chain'
    AFTER rotated_to_token_id;

-- Add last_rotation_at for tracking rotation frequency
ALTER TABLE refresh_tokens
    ADD COLUMN IF NOT EXISTS last_rotation_at TIMESTAMP NULL COMMENT 'Timestamp of the last token rotation'
    AFTER use_count;

-- Add rotation_count for security monitoring
ALTER TABLE refresh_tokens
    ADD COLUMN IF NOT EXISTS rotation_count INT UNSIGNED NOT NULL DEFAULT 0 COMMENT 'Number of times this token has been rotated'
    AFTER last_rotation_at;

-- Create indexes for new columns
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_token_family ON refresh_tokens(token_family);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_device_fingerprint ON refresh_tokens(device_fingerprint);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_previous_token_id ON refresh_tokens(previous_token_id);

-- Create token_blacklist table for immediate token revocation
CREATE TABLE IF NOT EXISTS token_blacklist (
    -- JWT ID (jti claim) as primary key
    jti VARCHAR(36) NOT NULL,
    
    -- Token metadata
    token_type ENUM('access', 'refresh') NOT NULL DEFAULT 'access',
    user_id CHAR(36) NULL,
    
    -- Blacklist lifecycle
    blacklisted_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NOT NULL,
    reason VARCHAR(255) NULL,
    
    -- Additional context
    ip_address VARCHAR(45) NULL,
    user_agent TEXT NULL,
    
    -- Audit fields
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Constraints
    PRIMARY KEY (jti),
    INDEX idx_blacklist_expires_at (expires_at),
    INDEX idx_blacklist_user_id (user_id),
    INDEX idx_blacklist_created_at (created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT = 'Blacklist for revoked JWT tokens - enables immediate token invalidation';

-- Add column comments for token_blacklist
ALTER TABLE token_blacklist
    MODIFY COLUMN jti VARCHAR(36) NOT NULL COMMENT 'JWT ID from the jti claim - unique identifier for the token',
    MODIFY COLUMN token_type ENUM('access', 'refresh') NOT NULL DEFAULT 'access' COMMENT 'Type of token being blacklisted',
    MODIFY COLUMN user_id CHAR(36) NULL COMMENT 'User ID associated with the blacklisted token',
    MODIFY COLUMN blacklisted_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When the token was added to blacklist',
    MODIFY COLUMN expires_at TIMESTAMP NOT NULL COMMENT 'When the blacklist entry can be removed (matches token expiry)',
    MODIFY COLUMN reason VARCHAR(255) NULL COMMENT 'Reason for blacklisting (e.g., "logout", "suspicious_activity", "password_change")',
    MODIFY COLUMN ip_address VARCHAR(45) NULL COMMENT 'IP address from which blacklist was triggered',
    MODIFY COLUMN user_agent TEXT NULL COMMENT 'User agent of the client that triggered blacklisting',
    MODIFY COLUMN created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Record creation timestamp';

-- Create stored procedure for enhanced token rotation
DELIMITER $$

CREATE PROCEDURE IF NOT EXISTS rotate_refresh_token(
    IN p_old_token_hash VARCHAR(64),
    IN p_new_token_hash VARCHAR(64),
    IN p_new_token_id CHAR(36),
    IN p_device_fingerprint VARCHAR(255)
)
BEGIN
    DECLARE v_user_id CHAR(36);
    DECLARE v_token_family VARCHAR(36);
    DECLARE v_old_token_id CHAR(36);
    DECLARE v_is_valid BOOLEAN;
    
    -- Start transaction for atomic operation
    START TRANSACTION;
    
    -- Find the old token and check validity
    SELECT id, user_id, token_family, 
           (is_revoked = FALSE AND expires_at > NOW()) AS is_valid
    INTO v_old_token_id, v_user_id, v_token_family, v_is_valid
    FROM refresh_tokens
    WHERE token_hash = p_old_token_hash
    FOR UPDATE;
    
    -- Check if token is valid
    IF v_is_valid = FALSE THEN
        -- If token is already revoked or expired, this might be a reuse attack
        -- Revoke entire token family for security
        IF v_token_family IS NOT NULL THEN
            UPDATE refresh_tokens
            SET is_revoked = TRUE,
                revoked_at = NOW(),
                revoke_reason = 'Token reuse detected - possible security breach'
            WHERE token_family = v_token_family
            AND is_revoked = FALSE;
        END IF;
        
        ROLLBACK;
        SIGNAL SQLSTATE '45000' 
        SET MESSAGE_TEXT = 'Invalid token - possible reuse attack detected';
    END IF;
    
    -- Generate new token family if not exists
    IF v_token_family IS NULL THEN
        SET v_token_family = UUID();
        
        -- Update old token with family ID
        UPDATE refresh_tokens
        SET token_family = v_token_family
        WHERE id = v_old_token_id;
    END IF;
    
    -- Mark old token as rotated
    UPDATE refresh_tokens
    SET rotated_to_token_id = p_new_token_id,
        is_revoked = TRUE,
        revoked_at = NOW(),
        revoke_reason = 'Token rotation',
        last_rotation_at = NOW(),
        rotation_count = rotation_count + 1
    WHERE id = v_old_token_id;
    
    -- Insert new token
    INSERT INTO refresh_tokens (
        id, user_id, token_hash, token_family,
        device_fingerprint, previous_token_id,
        rotated_from_token_id, expires_at,
        created_at
    ) VALUES (
        p_new_token_id,
        v_user_id,
        p_new_token_hash,
        v_token_family,
        p_device_fingerprint,
        v_old_token_id,
        v_old_token_id,
        DATE_ADD(NOW(), INTERVAL 30 DAY),
        NOW()
    );
    
    COMMIT;
END$$

DELIMITER ;

-- Create stored procedure for token family revocation
DELIMITER $$

CREATE PROCEDURE IF NOT EXISTS revoke_token_family(
    IN p_token_family VARCHAR(36),
    IN p_reason VARCHAR(255)
)
BEGIN
    UPDATE refresh_tokens
    SET is_revoked = TRUE,
        revoked_at = NOW(),
        revoke_reason = p_reason
    WHERE token_family = p_token_family
    AND is_revoked = FALSE;
    
    SELECT ROW_COUNT() AS revoked_count;
END$$

DELIMITER ;

-- Create stored procedure for blacklist cleanup
DELIMITER $$

CREATE PROCEDURE IF NOT EXISTS cleanup_token_blacklist()
BEGIN
    -- Delete expired blacklist entries
    DELETE FROM token_blacklist 
    WHERE expires_at < NOW();
    
    SELECT ROW_COUNT() AS deleted_count;
END$$

DELIMITER ;

-- Update the cleanup_expired_refresh_tokens procedure to handle new fields
DELIMITER $$

DROP PROCEDURE IF EXISTS cleanup_expired_refresh_tokens$$

CREATE PROCEDURE cleanup_expired_refresh_tokens()
BEGIN
    -- Delete expired tokens older than 7 days (grace period after expiry)
    DELETE FROM refresh_tokens 
    WHERE expires_at < DATE_SUB(NOW(), INTERVAL 7 DAY);
    
    -- Delete revoked tokens older than 30 days
    DELETE FROM refresh_tokens 
    WHERE is_revoked = TRUE 
    AND revoked_at < DATE_SUB(NOW(), INTERVAL 30 DAY);
    
    -- Cleanup orphaned token families (all tokens revoked)
    DELETE FROM refresh_tokens
    WHERE token_family IN (
        SELECT DISTINCT token_family 
        FROM (
            SELECT token_family
            FROM refresh_tokens
            WHERE token_family IS NOT NULL
            GROUP BY token_family
            HAVING COUNT(*) = SUM(CASE WHEN is_revoked = TRUE THEN 1 ELSE 0 END)
            AND MAX(revoked_at) < DATE_SUB(NOW(), INTERVAL 7 DAY)
        ) AS orphaned_families
    );
END$$

DELIMITER ;

-- Create event scheduler for automatic cleanup (runs daily at 3 AM)
CREATE EVENT IF NOT EXISTS auto_cleanup_tokens
ON SCHEDULE EVERY 1 DAY
STARTS (DATE(NOW()) + INTERVAL 1 DAY + INTERVAL 3 HOUR)
DO
BEGIN
    CALL cleanup_expired_refresh_tokens();
    CALL cleanup_token_blacklist();
END;

-- Enable event scheduler if not already enabled
SET GLOBAL event_scheduler = ON;

-- ============================================================================
-- DOWN MIGRATION
-- ============================================================================
-- To rollback this migration, uncomment and run:
-- DROP EVENT IF EXISTS auto_cleanup_tokens;
-- DROP PROCEDURE IF EXISTS cleanup_token_blacklist;
-- DROP PROCEDURE IF EXISTS revoke_token_family;
-- DROP PROCEDURE IF EXISTS rotate_refresh_token;
-- DROP PROCEDURE IF EXISTS cleanup_expired_refresh_tokens;
-- DROP TABLE IF EXISTS token_blacklist;
-- DROP INDEX IF EXISTS idx_refresh_tokens_previous_token_id ON refresh_tokens;
-- DROP INDEX IF EXISTS idx_refresh_tokens_device_fingerprint ON refresh_tokens;
-- DROP INDEX IF EXISTS idx_refresh_tokens_token_family ON refresh_tokens;
-- ALTER TABLE refresh_tokens
--     DROP COLUMN IF EXISTS rotation_count,
--     DROP COLUMN IF EXISTS last_rotation_at,
--     DROP COLUMN IF EXISTS previous_token_id,
--     DROP COLUMN IF EXISTS device_fingerprint,
--     DROP COLUMN IF EXISTS token_family;
-- DROP TABLE IF EXISTS verification_codes;