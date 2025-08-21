-- Migration: 003_create_user_profiles_table
-- Description: Create user profiles table for extended user information
-- Date: 2025-08-20

-- Create user_profiles table for extended user information
CREATE TABLE IF NOT EXISTS user_profiles (
    -- 基本信息
    user_id CHAR(36) NOT NULL COMMENT 'Reference to users.id',
    display_name VARCHAR(50) NULL COMMENT 'Display name for user',
    avatar_url VARCHAR(255) NULL COMMENT 'Profile picture URL',
    bio TEXT NULL COMMENT 'User biography/description',
    
    -- 联系方式
    email VARCHAR(100) NULL COMMENT 'Email address (optional)',
    wechat_id VARCHAR(50) NULL COMMENT 'WeChat ID for communication',
    
    -- 地址信息
    preferred_city VARCHAR(50) NULL COMMENT 'Preferred working city',
    address TEXT NULL COMMENT 'Detailed address',
    latitude DECIMAL(10,8) NULL COMMENT 'GPS latitude for location-based matching',
    longitude DECIMAL(11,8) NULL COMMENT 'GPS longitude for location-based matching',
    
    -- 时间戳
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    -- 约束
    PRIMARY KEY (user_id),
    CONSTRAINT fk_user_profiles_user_id 
        FOREIGN KEY (user_id) REFERENCES users(id) 
        ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Create indexes for performance optimization
CREATE INDEX idx_user_profiles_preferred_city ON user_profiles(preferred_city);
CREATE INDEX idx_user_profiles_location ON user_profiles(latitude, longitude);
CREATE INDEX idx_user_profiles_display_name ON user_profiles(display_name);

-- Add table comment
ALTER TABLE user_profiles COMMENT = 'Extended user profile information';

-- Add column comments
ALTER TABLE user_profiles
    MODIFY COLUMN user_id CHAR(36) NOT NULL COMMENT 'Reference to users.id',
    MODIFY COLUMN display_name VARCHAR(50) NULL COMMENT 'Display name shown to other users',
    MODIFY COLUMN avatar_url VARCHAR(255) NULL COMMENT 'URL to user profile picture',
    MODIFY COLUMN bio TEXT NULL COMMENT 'User self-description or biography',
    MODIFY COLUMN email VARCHAR(100) NULL COMMENT 'Optional email address for notifications',
    MODIFY COLUMN wechat_id VARCHAR(50) NULL COMMENT 'WeChat ID for customer-worker communication',
    MODIFY COLUMN preferred_city VARCHAR(50) NULL COMMENT 'City where user prefers to work/find services',
    MODIFY COLUMN address TEXT NULL COMMENT 'Detailed address information',
    MODIFY COLUMN latitude DECIMAL(10,8) NULL COMMENT 'GPS latitude coordinate for location services',
    MODIFY COLUMN longitude DECIMAL(11,8) NULL COMMENT 'GPS longitude coordinate for location services',
    MODIFY COLUMN created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Profile creation timestamp',
    MODIFY COLUMN updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Profile last update timestamp';