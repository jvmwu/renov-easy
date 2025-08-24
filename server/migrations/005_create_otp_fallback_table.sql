-- Migration: Create OTP Fallback Table for Redis Failure Scenarios
-- Purpose: Provides database fallback storage for OTP codes when Redis is unavailable
-- Requirements: 8.2 - Database fallback for Redis failures
-- Created: 2025-08-24
-- Security: Stores encrypted OTP data only (AES-256-GCM)

-- Create table for OTP fallback storage
CREATE TABLE IF NOT EXISTS otp_fallback (
    -- Primary key
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    
    -- Phone number (indexed for quick lookups)
    phone VARCHAR(20) NOT NULL,
    
    -- Encrypted OTP data (AES-256-GCM)
    ciphertext TEXT NOT NULL COMMENT 'Base64 encoded encrypted OTP',
    nonce VARCHAR(255) NOT NULL COMMENT 'Base64 encoded nonce for AES-GCM',
    key_id VARCHAR(100) NOT NULL COMMENT 'Encryption key identifier',
    
    -- Timestamps
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    expires_at TIMESTAMP(6) NOT NULL COMMENT 'OTP expiration time',
    
    -- Attempt tracking
    attempt_count INT UNSIGNED DEFAULT 0 COMMENT 'Number of verification attempts',
    
    -- Metadata
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6),
    
    -- Indexes
    INDEX idx_phone (phone),
    INDEX idx_expires_at (expires_at),
    INDEX idx_created_at (created_at),
    
    -- Unique constraint to prevent duplicate active OTPs
    UNIQUE KEY uk_phone_active (phone, expires_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT='Fallback storage for encrypted OTP codes when Redis is unavailable';

-- Create event to automatically clean up expired OTPs
DELIMITER $$
CREATE EVENT IF NOT EXISTS cleanup_expired_otps
ON SCHEDULE EVERY 1 HOUR
COMMENT 'Clean up expired OTP records from fallback table'
DO
BEGIN
    DELETE FROM otp_fallback 
    WHERE expires_at < NOW() 
    LIMIT 1000;
END$$
DELIMITER ;

-- Enable the event scheduler if not already enabled
SET GLOBAL event_scheduler = ON;

-- Add table statistics
ALTER TABLE otp_fallback
    COMMENT='Fallback storage for encrypted OTP codes when Redis is unavailable. Auto-cleanup runs hourly.';

-- Performance optimization: Add partition by day for better query performance and easier maintenance
-- Note: Uncomment below if expecting high volume
-- ALTER TABLE otp_fallback
-- PARTITION BY RANGE (TO_DAYS(expires_at)) (
--     PARTITION p0 VALUES LESS THAN (TO_DAYS('2025-01-01')),
--     PARTITION p1 VALUES LESS THAN (TO_DAYS('2025-02-01')),
--     PARTITION p2 VALUES LESS THAN (TO_DAYS('2025-03-01')),
--     PARTITION p3 VALUES LESS THAN (TO_DAYS('2025-04-01')),
--     PARTITION p4 VALUES LESS THAN (TO_DAYS('2025-05-01')),
--     PARTITION p5 VALUES LESS THAN (TO_DAYS('2025-06-01')),
--     PARTITION p6 VALUES LESS THAN (TO_DAYS('2025-07-01')),
--     PARTITION p7 VALUES LESS THAN (TO_DAYS('2025-08-01')),
--     PARTITION p8 VALUES LESS THAN (TO_DAYS('2025-09-01')),
--     PARTITION p9 VALUES LESS THAN (TO_DAYS('2025-10-01')),
--     PARTITION p10 VALUES LESS THAN (TO_DAYS('2025-11-01')),
--     PARTITION p11 VALUES LESS THAN (TO_DAYS('2025-12-01')),
--     PARTITION p_future VALUES LESS THAN MAXVALUE
-- );