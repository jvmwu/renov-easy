-- Migration: 009_create_system_tables
-- Description: Create system configuration and analytics tables
-- Date: 2025-08-20

-- Create system_configs table for application configuration
CREATE TABLE IF NOT EXISTS system_configs (
    config_key VARCHAR(100) NOT NULL COMMENT 'Configuration key name',
    config_value TEXT NOT NULL COMMENT 'Configuration value (supports JSON format)',
    description TEXT NULL COMMENT 'Human-readable description of this configuration',
    config_type ENUM('string', 'number', 'boolean', 'json') NOT NULL DEFAULT 'string' COMMENT 'Data type of the configuration value',
    is_sensitive BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether value contains sensitive data (passwords, keys)',
    
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    updated_by CHAR(36) NULL COMMENT 'Admin user who last updated this configuration',
    
    PRIMARY KEY (config_key)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Create indexes for system_configs
CREATE INDEX idx_system_configs_config_type ON system_configs(config_type);
CREATE INDEX idx_system_configs_updated_at ON system_configs(updated_at);

-- Create user_analytics table for behavioral tracking
CREATE TABLE IF NOT EXISTS user_analytics (
    id CHAR(36) NOT NULL COMMENT 'Analytics record ID',
    user_id CHAR(36) NULL COMMENT 'User ID (NULL for anonymous users)',
    session_id VARCHAR(100) NOT NULL COMMENT 'Session identifier for grouping events',
    
    -- 事件信息
    event_type VARCHAR(50) NOT NULL COMMENT 'Type of event (page_view, click, search, order_create, etc.)',
    event_category VARCHAR(50) NOT NULL COMMENT 'Event category (navigation, order, search, etc.)',
    event_action VARCHAR(100) NOT NULL COMMENT 'Specific action taken',
    event_label VARCHAR(200) NULL COMMENT 'Additional event context or label',
    event_value INT NULL COMMENT 'Numeric value associated with event',
    
    -- 页面/界面信息
    page_url VARCHAR(500) NULL COMMENT 'Page URL or mobile screen identifier',
    page_title VARCHAR(200) NULL COMMENT 'Page title or screen name',
    referrer_url VARCHAR(500) NULL COMMENT 'Previous page/screen URL',
    
    -- 设备信息
    user_agent TEXT NULL COMMENT 'User agent string for web, device info for mobile',
    ip_address VARCHAR(45) NULL COMMENT 'User IP address (IPv4 or IPv6)',
    device_type ENUM('mobile', 'tablet', 'desktop') NULL COMMENT 'Type of device used',
    os VARCHAR(50) NULL COMMENT 'Operating system (iOS, Android, Windows, etc.)',
    browser VARCHAR(50) NULL COMMENT 'Browser name (or mobile app version)',
    
    -- 位置信息
    city VARCHAR(50) NULL COMMENT 'User city (from IP or GPS)',
    country VARCHAR(50) NULL COMMENT 'User country',
    
    -- 时间信息
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    PRIMARY KEY (id),
    CONSTRAINT fk_user_analytics_user_id 
        FOREIGN KEY (user_id) REFERENCES users(id) 
        ON DELETE SET NULL ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Create indexes for user_analytics
CREATE INDEX idx_user_analytics_user_id ON user_analytics(user_id);
CREATE INDEX idx_user_analytics_session_id ON user_analytics(session_id);
CREATE INDEX idx_user_analytics_event_type ON user_analytics(event_type);
CREATE INDEX idx_user_analytics_created_at ON user_analytics(created_at);
CREATE INDEX idx_user_analytics_page_analysis ON user_analytics(page_url(100), created_at);
CREATE INDEX idx_user_analytics_event_analysis ON user_analytics(event_category, event_type, created_at);

-- Add table comments
ALTER TABLE system_configs COMMENT = 'System configuration settings';
ALTER TABLE user_analytics COMMENT = 'User behavior analytics and tracking';

-- Add column comments for system_configs
ALTER TABLE system_configs
    MODIFY COLUMN config_key VARCHAR(100) NOT NULL COMMENT 'Unique configuration identifier (e.g., max_images_per_order)',
    MODIFY COLUMN config_value TEXT NOT NULL COMMENT 'Configuration value, can be string, number, boolean, or JSON object',
    MODIFY COLUMN description TEXT NULL COMMENT 'Human-readable explanation of what this configuration controls',
    MODIFY COLUMN config_type ENUM('string', 'number', 'boolean', 'json') NOT NULL DEFAULT 'string' COMMENT 'Expected data type for proper parsing and validation',
    MODIFY COLUMN is_sensitive BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether this config contains secrets (API keys, passwords)',
    MODIFY COLUMN created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When configuration was first created',
    MODIFY COLUMN updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'When configuration was last modified',
    MODIFY COLUMN updated_by CHAR(36) NULL COMMENT 'Admin user ID who made the last update';

-- Add column comments for user_analytics
ALTER TABLE user_analytics
    MODIFY COLUMN id CHAR(36) NOT NULL COMMENT 'Unique analytics event identifier (UUID v4)',
    MODIFY COLUMN user_id CHAR(36) NULL COMMENT 'User who performed the action (NULL for anonymous)',
    MODIFY COLUMN session_id VARCHAR(100) NOT NULL COMMENT 'Session ID to group related user actions',
    MODIFY COLUMN event_type VARCHAR(50) NOT NULL COMMENT 'Type of user action (page_view, button_click, search, order_create)',
    MODIFY COLUMN event_category VARCHAR(50) NOT NULL COMMENT 'Broad category (navigation, order_management, search, communication)',
    MODIFY COLUMN event_action VARCHAR(100) NOT NULL COMMENT 'Specific action description (view_order_detail, send_message, create_bid)',
    MODIFY COLUMN event_label VARCHAR(200) NULL COMMENT 'Additional context (order_id, search_term, button_name)',
    MODIFY COLUMN event_value INT NULL COMMENT 'Numeric value when applicable (search_results_count, order_amount)',
    MODIFY COLUMN page_url VARCHAR(500) NULL COMMENT 'URL or mobile screen identifier where event occurred',
    MODIFY COLUMN page_title VARCHAR(200) NULL COMMENT 'Human-readable page or screen title',
    MODIFY COLUMN referrer_url VARCHAR(500) NULL COMMENT 'Previous page/screen that led to this event',
    MODIFY COLUMN user_agent TEXT NULL COMMENT 'Browser user agent string or mobile app version info',
    MODIFY COLUMN ip_address VARCHAR(45) NULL COMMENT 'User IP address for location and security analysis',
    MODIFY COLUMN device_type ENUM('mobile', 'tablet', 'desktop') NULL COMMENT 'Device category for responsive analytics',
    MODIFY COLUMN os VARCHAR(50) NULL COMMENT 'Operating system (iOS 16.1, Android 13, Windows 11)',
    MODIFY COLUMN browser VARCHAR(50) NULL COMMENT 'Browser name and version or mobile app identifier',
    MODIFY COLUMN city VARCHAR(50) NULL COMMENT 'User location city (derived from IP or GPS)',
    MODIFY COLUMN country VARCHAR(50) NULL COMMENT 'User location country',
    MODIFY COLUMN created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When the event occurred';

-- Insert initial system configurations
INSERT INTO system_configs (config_key, config_value, description, config_type) VALUES
('max_images_per_order', '10', 'Maximum number of images allowed per order', 'number'),
('order_auto_cancel_hours', '72', 'Hours before unpaid orders are automatically canceled', 'number'),
('worker_search_radius_km', '50', 'Default search radius in kilometers for finding workers', 'number'),
('min_bid_amount', '50', 'Minimum bid amount in CNY', 'number'),
('max_bid_amount', '50000', 'Maximum bid amount in CNY', 'number'),
('sms_rate_limit_per_hour', '3', 'Maximum SMS verification codes per phone number per hour', 'number'),
('review_required_for_completion', 'true', 'Whether customer must leave review to complete order', 'boolean'),
('worker_verification_required', 'false', 'Whether worker qualification verification is mandatory', 'boolean'),
('maintenance_mode', 'false', 'Whether application is in maintenance mode', 'boolean'),
('featured_cities', '["北京", "上海", "广州", "深圳", "杭州", "成都"]', 'List of featured cities for homepage', 'json'),
('notification_settings', '{"push_enabled": true, "email_enabled": false, "sms_enabled": true}', 'Default notification preferences for new users', 'json'),
('payment_methods', '["wechat_pay", "alipay"]', 'Supported payment methods', 'json'),
('app_version_min_supported', '1.0.0', 'Minimum app version that is supported', 'string'),
('api_rate_limit_per_minute', '60', 'Maximum API requests per minute per user', 'number'),
('image_max_size_mb', '5', 'Maximum image file size in megabytes', 'number');