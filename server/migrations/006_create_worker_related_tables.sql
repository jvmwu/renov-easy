-- Migration: 006_create_worker_related_tables  
-- Description: Create worker qualifications and statistics tables
-- Date: 2025-08-20

-- Create worker_qualifications table for professional certifications
CREATE TABLE IF NOT EXISTS worker_qualifications (
    id CHAR(36) NOT NULL COMMENT 'Unique qualification ID',
    worker_id CHAR(36) NOT NULL COMMENT 'Reference to users.id where user_type=worker',
    
    -- 资质信息
    license_number VARCHAR(100) NULL COMMENT 'Professional license number',
    license_type ENUM('general_contractor', 'electrical', 'plumbing', 'painting', 'flooring', 'tiling', 'carpentry', 'hvac', 'roofing') NOT NULL COMMENT 'Type of license/specialization',
    license_expiry DATE NULL COMMENT 'License expiration date',
    
    -- 认证状态
    is_verified BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether qualification is verified by admin',
    verified_at TIMESTAMP NULL COMMENT 'When verification was completed',
    verified_by CHAR(36) NULL COMMENT 'Admin user who verified this qualification',
    
    -- 证书图片
    certificate_url VARCHAR(255) NULL COMMENT 'URL to certificate image',
    
    -- 时间戳
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    -- 约束
    PRIMARY KEY (id),
    CONSTRAINT fk_worker_qualifications_worker_id 
        FOREIGN KEY (worker_id) REFERENCES users(id) 
        ON DELETE CASCADE ON UPDATE CASCADE,
    
    -- 一个工人在同一专业领域只能有一个资质
    UNIQUE KEY uk_worker_qualifications_worker_license (worker_id, license_type)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Create indexes for worker_qualifications
CREATE INDEX idx_worker_qualifications_worker_id ON worker_qualifications(worker_id);
CREATE INDEX idx_worker_qualifications_license_type ON worker_qualifications(license_type);
CREATE INDEX idx_worker_qualifications_is_verified ON worker_qualifications(is_verified);

-- Create worker_stats table for performance tracking
CREATE TABLE IF NOT EXISTS worker_stats (
    worker_id CHAR(36) NOT NULL COMMENT 'Reference to users.id',
    
    -- 订单统计
    total_orders INT NOT NULL DEFAULT 0 COMMENT 'Total completed orders',
    completed_orders INT NOT NULL DEFAULT 0 COMMENT 'Successfully completed orders',
    canceled_orders INT NOT NULL DEFAULT 0 COMMENT 'Canceled orders',
    
    -- 评分统计
    average_rating DECIMAL(3,2) NOT NULL DEFAULT 0.00 COMMENT 'Average customer rating (0-5.00)',
    total_reviews INT NOT NULL DEFAULT 0 COMMENT 'Total number of reviews received',
    
    -- 收入统计
    total_earnings DECIMAL(12,2) NOT NULL DEFAULT 0.00 COMMENT 'Total lifetime earnings in CNY',
    this_month_earnings DECIMAL(12,2) NOT NULL DEFAULT 0.00 COMMENT 'Current month earnings in CNY',
    
    -- 时间统计
    total_work_hours INT NOT NULL DEFAULT 0 COMMENT 'Total work hours completed',
    response_time_avg INT NOT NULL DEFAULT 0 COMMENT 'Average response time to messages in minutes',
    
    -- 时间戳
    last_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    -- 约束
    PRIMARY KEY (worker_id),
    CONSTRAINT fk_worker_stats_worker_id 
        FOREIGN KEY (worker_id) REFERENCES users(id) 
        ON DELETE CASCADE ON UPDATE CASCADE,
        
    -- 检查约束
    CHECK (average_rating >= 0 AND average_rating <= 5),
    CHECK (total_earnings >= 0),
    CHECK (this_month_earnings >= 0),
    CHECK (completed_orders <= total_orders),
    CHECK (total_work_hours >= 0),
    CHECK (response_time_avg >= 0)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Create indexes for worker_stats  
CREATE INDEX idx_worker_stats_average_rating ON worker_stats(average_rating DESC);
CREATE INDEX idx_worker_stats_total_orders ON worker_stats(total_orders DESC);
CREATE INDEX idx_worker_stats_rating_performance ON worker_stats(average_rating DESC, total_orders DESC);

-- Add table comments
ALTER TABLE worker_qualifications COMMENT = 'Worker professional qualifications and certifications';
ALTER TABLE worker_stats COMMENT = 'Worker performance and earnings statistics';

-- Add column comments for worker_qualifications
ALTER TABLE worker_qualifications
    MODIFY COLUMN id CHAR(36) NOT NULL COMMENT 'Unique qualification record identifier (UUID v4)',
    MODIFY COLUMN worker_id CHAR(36) NOT NULL COMMENT 'User ID of the worker (must have user_type=worker)',
    MODIFY COLUMN license_number VARCHAR(100) NULL COMMENT 'Official license or certification number',
    MODIFY COLUMN license_type ENUM('general_contractor', 'electrical', 'plumbing', 'painting', 'flooring', 'tiling', 'carpentry', 'hvac', 'roofing') NOT NULL COMMENT 'Professional specialization area',
    MODIFY COLUMN license_expiry DATE NULL COMMENT 'When the professional license expires',
    MODIFY COLUMN is_verified BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether admin has verified this qualification',
    MODIFY COLUMN verified_at TIMESTAMP NULL COMMENT 'Timestamp when verification was completed',
    MODIFY COLUMN verified_by CHAR(36) NULL COMMENT 'Admin user ID who completed the verification',
    MODIFY COLUMN certificate_url VARCHAR(255) NULL COMMENT 'URL to uploaded certificate/license image',
    MODIFY COLUMN created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When qualification was added',
    MODIFY COLUMN updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'When qualification was last updated';

-- Add column comments for worker_stats
ALTER TABLE worker_stats
    MODIFY COLUMN worker_id CHAR(36) NOT NULL COMMENT 'User ID of the worker',
    MODIFY COLUMN total_orders INT NOT NULL DEFAULT 0 COMMENT 'Total number of orders taken by this worker',
    MODIFY COLUMN completed_orders INT NOT NULL DEFAULT 0 COMMENT 'Number of orders successfully completed',
    MODIFY COLUMN canceled_orders INT NOT NULL DEFAULT 0 COMMENT 'Number of orders that were canceled',
    MODIFY COLUMN average_rating DECIMAL(3,2) NOT NULL DEFAULT 0.00 COMMENT 'Average rating from customer reviews (0.00-5.00)',
    MODIFY COLUMN total_reviews INT NOT NULL DEFAULT 0 COMMENT 'Total number of customer reviews received',
    MODIFY COLUMN total_earnings DECIMAL(12,2) NOT NULL DEFAULT 0.00 COMMENT 'Lifetime total earnings in Chinese Yuan',
    MODIFY COLUMN this_month_earnings DECIMAL(12,2) NOT NULL DEFAULT 0.00 COMMENT 'Earnings for the current month',
    MODIFY COLUMN total_work_hours INT NOT NULL DEFAULT 0 COMMENT 'Total hours spent working on completed orders',
    MODIFY COLUMN response_time_avg INT NOT NULL DEFAULT 0 COMMENT 'Average time to respond to customer messages (minutes)',
    MODIFY COLUMN last_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'When these statistics were last updated';