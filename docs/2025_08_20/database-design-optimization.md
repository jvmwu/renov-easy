# RenovEasy 数据库设计与优化方案

## 1. 项目概述

RenovEasy (装修易) 是一个家庭装修服务市场平台，连接需要装修服务的客户与专业装修工人。基于现有的认证系统，需要扩展完整的业务数据模型。

## 2. 当前数据库状态分析

### 2.1 现有表结构
- `users` - 用户基础信息（客户和工人）
- `refresh_tokens` - JWT令牌管理
- `auth_audit_log` - 认证安全审计

### 2.2 数据库技术栈
- **数据库**: MySQL 8.0
- **连接池**: SQLx with connection pooling
- **缓存**: Redis 7.0
- **架构**: Clean Architecture with Repository Pattern

## 3. 完整表结构设计

### 3.1 用户相关表

#### 用户档案扩展表 (user_profiles)
```sql
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
        ON DELETE CASCADE ON UPDATE CASCADE,
    
    -- 索引
    INDEX idx_preferred_city (preferred_city),
    INDEX idx_location (latitude, longitude),
    INDEX idx_display_name (display_name)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT='Extended user profile information';
```

#### 工人资质表 (worker_qualifications)
```sql
CREATE TABLE IF NOT EXISTS worker_qualifications (
    id CHAR(36) NOT NULL COMMENT 'Unique qualification ID',
    worker_id CHAR(36) NOT NULL COMMENT 'Reference to users.id where user_type=worker',
    
    -- 资质信息
    license_number VARCHAR(100) NULL COMMENT 'Professional license number',
    license_type ENUM('general_contractor', 'electrical', 'plumbing', 'painting', 'flooring', 'tiling', 'carpentry') NOT NULL COMMENT 'Type of license/specialization',
    license_expiry DATE NULL COMMENT 'License expiration date',
    
    -- 认证状态
    is_verified BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether qualification is verified',
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
    
    -- 索引
    INDEX idx_worker_id (worker_id),
    INDEX idx_license_type (license_type),
    INDEX idx_is_verified (is_verified),
    UNIQUE KEY uk_worker_license (worker_id, license_type)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT='Worker professional qualifications and certifications';
```

#### 工人统计表 (worker_stats)
```sql
CREATE TABLE IF NOT EXISTS worker_stats (
    worker_id CHAR(36) NOT NULL COMMENT 'Reference to users.id',
    
    -- 订单统计
    total_orders INT NOT NULL DEFAULT 0 COMMENT 'Total completed orders',
    completed_orders INT NOT NULL DEFAULT 0 COMMENT 'Successfully completed orders',
    canceled_orders INT NOT NULL DEFAULT 0 COMMENT 'Canceled orders',
    
    -- 评分统计
    average_rating DECIMAL(3,2) NOT NULL DEFAULT 0.00 COMMENT 'Average customer rating (0-5.00)',
    total_reviews INT NOT NULL DEFAULT 0 COMMENT 'Total number of reviews',
    
    -- 收入统计
    total_earnings DECIMAL(12,2) NOT NULL DEFAULT 0.00 COMMENT 'Total earnings in CNY',
    this_month_earnings DECIMAL(12,2) NOT NULL DEFAULT 0.00 COMMENT 'Current month earnings',
    
    -- 时间统计
    total_work_hours INT NOT NULL DEFAULT 0 COMMENT 'Total work hours completed',
    response_time_avg INT NOT NULL DEFAULT 0 COMMENT 'Average response time in minutes',
    
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
    CHECK (completed_orders <= total_orders)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT='Worker performance and earnings statistics';
```

### 3.2 订单相关表

#### 服务分类表 (service_categories)
```sql
CREATE TABLE IF NOT EXISTS service_categories (
    id INT AUTO_INCREMENT NOT NULL COMMENT 'Category ID',
    name VARCHAR(50) NOT NULL COMMENT 'Category name',
    name_en VARCHAR(50) NOT NULL COMMENT 'Category name in English',
    parent_id INT NULL COMMENT 'Parent category for hierarchical structure',
    icon_url VARCHAR(255) NULL COMMENT 'Category icon URL',
    description TEXT NULL COMMENT 'Category description',
    is_active BOOLEAN NOT NULL DEFAULT TRUE COMMENT 'Whether category is active',
    sort_order INT NOT NULL DEFAULT 0 COMMENT 'Display order',
    
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    PRIMARY KEY (id),
    CONSTRAINT fk_service_categories_parent 
        FOREIGN KEY (parent_id) REFERENCES service_categories(id) 
        ON DELETE SET NULL ON UPDATE CASCADE,
        
    INDEX idx_parent_id (parent_id),
    INDEX idx_is_active (is_active),
    INDEX idx_sort_order (sort_order)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT='Hierarchical service categories';

-- 插入初始数据
INSERT INTO service_categories (name, name_en, description, sort_order) VALUES
('室内装修', 'Interior Decoration', '室内装修和装饰服务', 1),
('水电维修', 'Plumbing & Electrical', '水管和电路维修服务', 2),
('家具安装', 'Furniture Installation', '家具组装和安装服务', 3),
('墙面处理', 'Wall Treatment', '刷漆、贴壁纸等墙面处理', 4),
('地板铺设', 'Flooring Installation', '木地板、瓷砖等地面铺设', 5);
```

#### 订单主表 (orders)
```sql
CREATE TABLE IF NOT EXISTS orders (
    -- 基本信息
    id CHAR(36) NOT NULL COMMENT 'Unique order ID (UUID)',
    order_number VARCHAR(20) NOT NULL COMMENT 'Human-readable order number',
    customer_id CHAR(36) NOT NULL COMMENT 'Customer user ID',
    worker_id CHAR(36) NULL COMMENT 'Assigned worker user ID',
    
    -- 服务信息
    service_category_id INT NOT NULL COMMENT 'Service category',
    title VARCHAR(100) NOT NULL COMMENT 'Order title/summary',
    description TEXT NOT NULL COMMENT 'Detailed service requirements',
    
    -- 位置信息
    address TEXT NOT NULL COMMENT 'Service address',
    latitude DECIMAL(10,8) NOT NULL COMMENT 'Service location latitude',
    longitude DECIMAL(11,8) NOT NULL COMMENT 'Service location longitude',
    city VARCHAR(50) NOT NULL COMMENT 'City name',
    district VARCHAR(50) NULL COMMENT 'District/area name',
    
    -- 预算和支付
    budget_min DECIMAL(10,2) NOT NULL COMMENT 'Minimum budget (CNY)',
    budget_max DECIMAL(10,2) NOT NULL COMMENT 'Maximum budget (CNY)',
    final_price DECIMAL(10,2) NULL COMMENT 'Final agreed price',
    
    -- 时间安排
    preferred_start_date DATE NULL COMMENT 'Customer preferred start date',
    estimated_duration INT NULL COMMENT 'Estimated duration in days',
    actual_start_date DATE NULL COMMENT 'Actual work start date',
    actual_end_date DATE NULL COMMENT 'Actual completion date',
    
    -- 订单状态
    status ENUM('draft', 'published', 'assigned', 'in_progress', 'completed', 'canceled', 'disputed') 
        NOT NULL DEFAULT 'draft' COMMENT 'Order status',
    priority ENUM('low', 'normal', 'high', 'urgent') NOT NULL DEFAULT 'normal' COMMENT 'Order priority',
    
    -- 时间戳
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    published_at TIMESTAMP NULL COMMENT 'When order was published',
    assigned_at TIMESTAMP NULL COMMENT 'When worker was assigned',
    completed_at TIMESTAMP NULL COMMENT 'When order was completed',
    canceled_at TIMESTAMP NULL COMMENT 'When order was canceled',
    
    -- 取消信息
    cancellation_reason ENUM('customer_request', 'worker_unavailable', 'price_disagreement', 'other') NULL,
    cancellation_note TEXT NULL COMMENT 'Cancellation explanation',
    
    -- 约束
    PRIMARY KEY (id),
    UNIQUE KEY uk_order_number (order_number),
    
    CONSTRAINT fk_orders_customer_id 
        FOREIGN KEY (customer_id) REFERENCES users(id) 
        ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT fk_orders_worker_id 
        FOREIGN KEY (worker_id) REFERENCES users(id) 
        ON DELETE SET NULL ON UPDATE CASCADE,
    CONSTRAINT fk_orders_service_category 
        FOREIGN KEY (service_category_id) REFERENCES service_categories(id) 
        ON DELETE RESTRICT ON UPDATE CASCADE,
        
    -- 业务约束
    CHECK (budget_min > 0 AND budget_max >= budget_min),
    CHECK (final_price IS NULL OR final_price > 0),
    CHECK (estimated_duration IS NULL OR estimated_duration > 0),
    
    -- 索引设计
    INDEX idx_customer_id (customer_id),
    INDEX idx_worker_id (worker_id),
    INDEX idx_status (status),
    INDEX idx_service_category (service_category_id),
    INDEX idx_location (city, latitude, longitude),
    INDEX idx_budget_range (budget_min, budget_max),
    INDEX idx_created_at (created_at),
    INDEX idx_published_at (published_at),
    INDEX idx_priority_status (priority, status),
    
    -- 复合索引用于地理位置搜索
    INDEX idx_geo_search (city, status, service_category_id),
    INDEX idx_worker_orders (worker_id, status, created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT='Main orders table for renovation services';
```

#### 订单图片表 (order_images)
```sql
CREATE TABLE IF NOT EXISTS order_images (
    id CHAR(36) NOT NULL COMMENT 'Image ID',
    order_id CHAR(36) NOT NULL COMMENT 'Reference to orders.id',
    image_url VARCHAR(500) NOT NULL COMMENT 'Image storage URL',
    image_type ENUM('requirement', 'before', 'progress', 'after') NOT NULL COMMENT 'Image type',
    caption TEXT NULL COMMENT 'Image description',
    uploaded_by CHAR(36) NOT NULL COMMENT 'User who uploaded the image',
    file_size INT NULL COMMENT 'File size in bytes',
    mime_type VARCHAR(50) NULL COMMENT 'MIME type of the image',
    
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    PRIMARY KEY (id),
    CONSTRAINT fk_order_images_order_id 
        FOREIGN KEY (order_id) REFERENCES orders(id) 
        ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT fk_order_images_uploaded_by 
        FOREIGN KEY (uploaded_by) REFERENCES users(id) 
        ON DELETE CASCADE ON UPDATE CASCADE,
        
    INDEX idx_order_id (order_id),
    INDEX idx_image_type (image_type),
    INDEX idx_uploaded_by (uploaded_by)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT='Images associated with orders';
```

#### 订单竞价表 (order_bids)
```sql
CREATE TABLE IF NOT EXISTS order_bids (
    id CHAR(36) NOT NULL COMMENT 'Bid ID',
    order_id CHAR(36) NOT NULL COMMENT 'Reference to orders.id',
    worker_id CHAR(36) NOT NULL COMMENT 'Bidding worker ID',
    
    -- 竞价信息
    bid_price DECIMAL(10,2) NOT NULL COMMENT 'Bid price in CNY',
    estimated_days INT NOT NULL COMMENT 'Estimated completion time',
    message TEXT NULL COMMENT 'Bid message/proposal',
    
    -- 状态
    status ENUM('pending', 'accepted', 'rejected', 'withdrawn') NOT NULL DEFAULT 'pending',
    
    -- 时间戳
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    responded_at TIMESTAMP NULL COMMENT 'When customer responded',
    
    PRIMARY KEY (id),
    CONSTRAINT fk_order_bids_order_id 
        FOREIGN KEY (order_id) REFERENCES orders(id) 
        ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT fk_order_bids_worker_id 
        FOREIGN KEY (worker_id) REFERENCES users(id) 
        ON DELETE CASCADE ON UPDATE CASCADE,
        
    -- 一个工人对一个订单只能有一个竞价
    UNIQUE KEY uk_order_worker_bid (order_id, worker_id),
    
    INDEX idx_order_id (order_id),
    INDEX idx_worker_id (worker_id),
    INDEX idx_status (status),
    INDEX idx_created_at (created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT='Worker bids for orders';
```

### 3.3 评价和交流表

#### 评价表 (reviews)
```sql
CREATE TABLE IF NOT EXISTS reviews (
    id CHAR(36) NOT NULL COMMENT 'Review ID',
    order_id CHAR(36) NOT NULL COMMENT 'Reference to completed order',
    reviewer_id CHAR(36) NOT NULL COMMENT 'User who wrote the review',
    reviewee_id CHAR(36) NOT NULL COMMENT 'User being reviewed',
    
    -- 评分 (1-5星)
    overall_rating TINYINT NOT NULL COMMENT 'Overall rating (1-5)',
    quality_rating TINYINT NOT NULL COMMENT 'Work quality rating',
    communication_rating TINYINT NOT NULL COMMENT 'Communication rating',
    punctuality_rating TINYINT NOT NULL COMMENT 'Punctuality rating',
    
    -- 评价内容
    review_text TEXT NULL COMMENT 'Written review',
    
    -- 状态
    is_public BOOLEAN NOT NULL DEFAULT TRUE COMMENT 'Whether review is public',
    is_verified BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether review is verified',
    
    -- 时间戳
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    PRIMARY KEY (id),
    CONSTRAINT fk_reviews_order_id 
        FOREIGN KEY (order_id) REFERENCES orders(id) 
        ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT fk_reviews_reviewer_id 
        FOREIGN KEY (reviewer_id) REFERENCES users(id) 
        ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT fk_reviews_reviewee_id 
        FOREIGN KEY (reviewee_id) REFERENCES users(id) 
        ON DELETE CASCADE ON UPDATE CASCADE,
        
    -- 每个订单只能有一个评价
    UNIQUE KEY uk_order_review (order_id),
    
    -- 评分约束
    CHECK (overall_rating BETWEEN 1 AND 5),
    CHECK (quality_rating BETWEEN 1 AND 5),
    CHECK (communication_rating BETWEEN 1 AND 5),
    CHECK (punctuality_rating BETWEEN 1 AND 5),
    
    INDEX idx_reviewee_id (reviewee_id),
    INDEX idx_overall_rating (overall_rating),
    INDEX idx_is_public (is_public),
    INDEX idx_created_at (created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT='Customer reviews for completed orders';
```

#### 消息表 (messages)
```sql
CREATE TABLE IF NOT EXISTS messages (
    id CHAR(36) NOT NULL COMMENT 'Message ID',
    conversation_id CHAR(36) NOT NULL COMMENT 'Conversation/chat ID',
    sender_id CHAR(36) NOT NULL COMMENT 'Message sender',
    receiver_id CHAR(36) NOT NULL COMMENT 'Message receiver',
    order_id CHAR(36) NULL COMMENT 'Related order (if applicable)',
    
    -- 消息内容
    message_type ENUM('text', 'image', 'location', 'system') NOT NULL DEFAULT 'text',
    content TEXT NOT NULL COMMENT 'Message content',
    attachment_url VARCHAR(500) NULL COMMENT 'Attachment URL for images',
    
    -- 状态
    is_read BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether message has been read',
    read_at TIMESTAMP NULL COMMENT 'When message was read',
    
    -- 时间戳
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    PRIMARY KEY (id),
    CONSTRAINT fk_messages_sender_id 
        FOREIGN KEY (sender_id) REFERENCES users(id) 
        ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT fk_messages_receiver_id 
        FOREIGN KEY (receiver_id) REFERENCES users(id) 
        ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT fk_messages_order_id 
        FOREIGN KEY (order_id) REFERENCES orders(id) 
        ON DELETE SET NULL ON UPDATE CASCADE,
        
    INDEX idx_conversation_id (conversation_id),
    INDEX idx_sender_id (sender_id),
    INDEX idx_receiver_id (receiver_id),
    INDEX idx_order_id (order_id),
    INDEX idx_created_at (created_at),
    INDEX idx_is_read (is_read),
    
    -- 复合索引用于会话查询
    INDEX idx_conversation_time (conversation_id, created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT='Messages between users';
```

### 3.4 收藏和通知表

#### 收藏表 (favorites)
```sql
CREATE TABLE IF NOT EXISTS favorites (
    id CHAR(36) NOT NULL COMMENT 'Favorite ID',
    user_id CHAR(36) NOT NULL COMMENT 'User who favorited',
    favorited_user_id CHAR(36) NOT NULL COMMENT 'User being favorited (worker)',
    
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    PRIMARY KEY (id),
    CONSTRAINT fk_favorites_user_id 
        FOREIGN KEY (user_id) REFERENCES users(id) 
        ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT fk_favorites_favorited_user_id 
        FOREIGN KEY (favorited_user_id) REFERENCES users(id) 
        ON DELETE CASCADE ON UPDATE CASCADE,
        
    -- 防止重复收藏
    UNIQUE KEY uk_user_favorite (user_id, favorited_user_id),
    
    INDEX idx_user_id (user_id),
    INDEX idx_favorited_user_id (favorited_user_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT='User favorites (customers favoriting workers)';
```

#### 通知表 (notifications)
```sql
CREATE TABLE IF NOT EXISTS notifications (
    id CHAR(36) NOT NULL COMMENT 'Notification ID',
    user_id CHAR(36) NOT NULL COMMENT 'Recipient user ID',
    
    -- 通知内容
    type ENUM('order_assigned', 'new_bid', 'order_completed', 'review_received', 'message_received', 'system') 
        NOT NULL COMMENT 'Notification type',
    title VARCHAR(100) NOT NULL COMMENT 'Notification title',
    message TEXT NOT NULL COMMENT 'Notification message',
    
    -- 关联数据
    related_order_id CHAR(36) NULL COMMENT 'Related order ID',
    related_user_id CHAR(36) NULL COMMENT 'Related user ID',
    
    -- 状态
    is_read BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether notification has been read',
    is_sent BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether push notification was sent',
    
    -- 时间戳
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    read_at TIMESTAMP NULL COMMENT 'When notification was read',
    sent_at TIMESTAMP NULL COMMENT 'When push notification was sent',
    
    PRIMARY KEY (id),
    CONSTRAINT fk_notifications_user_id 
        FOREIGN KEY (user_id) REFERENCES users(id) 
        ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT fk_notifications_related_order_id 
        FOREIGN KEY (related_order_id) REFERENCES orders(id) 
        ON DELETE SET NULL ON UPDATE CASCADE,
    CONSTRAINT fk_notifications_related_user_id 
        FOREIGN KEY (related_user_id) REFERENCES users(id) 
        ON DELETE SET NULL ON UPDATE CASCADE,
        
    INDEX idx_user_id (user_id),
    INDEX idx_type (type),
    INDEX idx_is_read (is_read),
    INDEX idx_created_at (created_at),
    INDEX idx_user_unread (user_id, is_read, created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT='User notifications';
```

### 3.5 系统配置和日志表

#### 系统配置表 (system_configs)
```sql
CREATE TABLE IF NOT EXISTS system_configs (
    config_key VARCHAR(100) NOT NULL COMMENT 'Configuration key',
    config_value TEXT NOT NULL COMMENT 'Configuration value (JSON format)',
    description TEXT NULL COMMENT 'Configuration description',
    config_type ENUM('string', 'number', 'boolean', 'json') NOT NULL DEFAULT 'string',
    is_sensitive BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether value contains sensitive data',
    
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    updated_by CHAR(36) NULL COMMENT 'Admin user who last updated this config',
    
    PRIMARY KEY (config_key),
    INDEX idx_config_type (config_type),
    INDEX idx_updated_at (updated_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT='System configuration settings';

-- 插入初始配置
INSERT INTO system_configs (config_key, config_value, description, config_type) VALUES
('max_images_per_order', '10', 'Maximum number of images per order', 'number'),
('order_auto_cancel_hours', '72', 'Hours before unpaid orders are auto-canceled', 'number'),
('worker_search_radius_km', '50', 'Default search radius for finding workers', 'number'),
('min_bid_amount', '50', 'Minimum bid amount in CNY', 'number'),
('max_bid_amount', '50000', 'Maximum bid amount in CNY', 'number');
```

## 4. 索引策略和查询优化

### 4.1 核心查询模式分析

#### 高频查询场景：
1. **客户搜索附近工人**
   ```sql
   -- 地理位置 + 服务类型搜索
   SELECT u.id, up.display_name, ws.average_rating
   FROM users u
   JOIN user_profiles up ON u.id = up.user_id
   JOIN worker_stats ws ON u.id = ws.worker_id
   WHERE u.user_type = 'worker'
     AND u.is_blocked = FALSE
     AND up.preferred_city = '北京'
     AND (up.latitude BETWEEN ? AND ?)
     AND (up.longitude BETWEEN ? AND ?)
   ORDER BY ws.average_rating DESC, ws.total_orders DESC
   LIMIT 20;
   ```

2. **工人查看附近订单**
   ```sql
   -- 地理位置 + 状态过滤
   SELECT o.id, o.title, o.budget_max, o.created_at,
          (6371 * acos(cos(radians(?)) * cos(radians(latitude)) * 
           cos(radians(longitude) - radians(?)) + sin(radians(?)) * 
           sin(radians(latitude)))) as distance
   FROM orders o
   WHERE o.status = 'published'
     AND o.city = ?
     AND o.service_category_id IN (?)
   HAVING distance < ?
   ORDER BY distance ASC, o.created_at DESC;
   ```

### 4.2 索引优化策略

#### 复合索引设计
```sql
-- 订单地理搜索优化
ALTER TABLE orders 
ADD INDEX idx_geo_search_optimized (status, city, service_category_id, published_at);

-- 工人搜索优化  
ALTER TABLE user_profiles 
ADD INDEX idx_worker_location (preferred_city, latitude, longitude);

-- 用户统计查询优化
ALTER TABLE worker_stats 
ADD INDEX idx_rating_performance (average_rating DESC, total_orders DESC);

-- 消息查询优化
ALTER TABLE messages 
ADD INDEX idx_conversation_unread (conversation_id, is_read, created_at DESC);

-- 通知查询优化
ALTER TABLE notifications 
ADD INDEX idx_user_notifications (user_id, is_read, created_at DESC);
```

### 4.3 查询性能优化

#### 分区策略
```sql
-- 按创建时间对订单表进行分区（适用于数据量大的情况）
ALTER TABLE orders PARTITION BY RANGE (YEAR(created_at)) (
    PARTITION p2024 VALUES LESS THAN (2025),
    PARTITION p2025 VALUES LESS THAN (2026),
    PARTITION p2026 VALUES LESS THAN (2027),
    PARTITION p_future VALUES LESS THAN MAXVALUE
);

-- 按用户ID哈希分区消息表（适用于高并发消息）
ALTER TABLE messages PARTITION BY HASH(CONV(SUBSTRING(sender_id, 1, 8), 16, 10)) PARTITIONS 8;
```

#### 查询优化建议
```sql
-- 1. 使用覆盖索引减少回表查询
CREATE INDEX idx_order_list_covering ON orders 
(customer_id, status, created_at DESC) 
INCLUDE (id, title, budget_max);

-- 2. 优化统计查询
CREATE INDEX idx_worker_stats_summary ON worker_stats 
(average_rating, total_orders, total_earnings);

-- 3. 消息分页查询优化
CREATE INDEX idx_message_pagination ON messages 
(conversation_id, created_at DESC, id);
```

## 5. 缓存策略（Redis）

### 5.1 缓存数据结构设计

#### 用户缓存
```redis
# 用户基本信息缓存 (TTL: 1小时)
user:profile:{user_id} -> JSON
{
  "id": "uuid",
  "display_name": "张师傅",
  "avatar_url": "https://...",
  "user_type": "worker",
  "preferred_city": "北京",
  "latitude": 39.9042,
  "longitude": 116.4074
}

# 用户会话缓存 (TTL: 15分钟，随访问更新)
user:session:{user_id} -> JSON
{
  "last_active": "2025-08-20T10:30:00Z",
  "device_info": "iPhone 15 Pro",
  "location": {"lat": 39.9042, "lng": 116.4074}
}
```

#### 地理位置缓存
```redis
# 使用Redis GEO数据结构存储工人位置 (TTL: 30分钟)
workers:location:beijing -> GEO
GEOADD workers:location:beijing 116.4074 39.9042 worker_id1
GEOADD workers:location:beijing 116.3974 39.9142 worker_id2

# 订单地理位置缓存
orders:location:published -> GEO
GEOADD orders:location:published 116.4074 39.9042 order_id1
```

#### 热门数据缓存
```redis
# 热门工人列表 (TTL: 10分钟)
workers:popular:{city} -> ZSET
ZADD workers:popular:beijing 4.8 worker_id1
ZADD workers:popular:beijing 4.7 worker_id2

# 订单统计缓存 (TTL: 5分钟)
stats:orders:daily:{date} -> HASH
HSET stats:orders:daily:2025-08-20 total 156 completed 98 in_progress 45
```

#### 实时数据缓存
```redis
# 未读消息计数 (实时更新)
user:unread_messages:{user_id} -> STRING
SET user:unread_messages:user123 5

# 在线用户列表 (TTL: 5分钟)
users:online -> SET
SADD users:online user_id1 user_id2 user_id3

# 订单竞价缓存 (TTL: 1小时)
order:bids:{order_id} -> LIST
LPUSH order:bids:order123 '{"worker_id":"w1","price":1000,"time":"..."}'
```

### 5.2 缓存更新策略

#### Write-Through 模式 (用户资料)
```rust
// 用户更新资料时同时更新数据库和缓存
async fn update_user_profile(user_id: &str, profile: UserProfile) -> Result<()> {
    // 1. 更新数据库
    let result = db_repository.update_user_profile(user_id, &profile).await?;
    
    // 2. 更新缓存
    let cache_key = format!("user:profile:{}", user_id);
    redis_client.set_json(&cache_key, &profile, Duration::hours(1)).await?;
    
    Ok(result)
}
```

#### Write-Behind 模式 (统计数据)
```rust
// 统计数据先写缓存，定期批量写入数据库
async fn increment_worker_stats(worker_id: &str, stat_type: StatType) {
    let cache_key = format!("stats:worker:{}:{}", worker_id, stat_type.as_str());
    redis_client.increment(&cache_key).await;
    
    // 添加到待更新队列
    let queue_key = "stats:update_queue";
    redis_client.lpush(&queue_key, &format!("{}:{}", worker_id, stat_type.as_str())).await;
}
```

### 5.3 缓存预热和失效策略

#### 缓存预热
```rust
// 应用启动时预热热门数据
async fn warm_up_cache() -> Result<()> {
    // 1. 预热热门城市的工人数据
    let popular_cities = vec!["北京", "上海", "广州", "深圳"];
    for city in popular_cities {
        let workers = db_repository.get_popular_workers(city, 50).await?;
        let cache_key = format!("workers:popular:{}", city);
        redis_client.set_zset(&cache_key, &workers, Duration::minutes(10)).await?;
    }
    
    // 2. 预热服务分类数据
    let categories = db_repository.get_service_categories().await?;
    redis_client.set_json("service:categories", &categories, Duration::hours(24)).await?;
    
    Ok(())
}
```

## 6. 数据一致性和事务边界

### 6.1 事务边界设计

#### 订单处理事务
```sql
-- 订单分配事务（强一致性要求）
START TRANSACTION;

-- 1. 检查订单状态
SELECT status FROM orders WHERE id = ? FOR UPDATE;

-- 2. 更新订单状态和工人分配
UPDATE orders 
SET status = 'assigned', worker_id = ?, assigned_at = NOW()
WHERE id = ? AND status = 'published';

-- 3. 拒绝其他竞价
UPDATE order_bids 
SET status = 'rejected', responded_at = NOW()
WHERE order_id = ? AND worker_id != ? AND status = 'pending';

-- 4. 接受选中的竞价
UPDATE order_bids 
SET status = 'accepted', responded_at = NOW()
WHERE order_id = ? AND worker_id = ?;

-- 5. 创建通知
INSERT INTO notifications (id, user_id, type, title, message, related_order_id)
VALUES 
(UUID(), ?, 'order_assigned', '订单已分配', '您的订单已分配给工人', ?),
(UUID(), ?, 'bid_accepted', '竞价被接受', '您的竞价已被客户接受', ?);

COMMIT;
```

#### 评价处理事务
```sql
-- 订单完成和评价事务
START TRANSACTION;

-- 1. 更新订单状态
UPDATE orders 
SET status = 'completed', completed_at = NOW(), final_price = ?
WHERE id = ? AND status = 'in_progress';

-- 2. 创建评价记录
INSERT INTO reviews (id, order_id, reviewer_id, reviewee_id, overall_rating, ...)
VALUES (UUID(), ?, ?, ?, ?, ...);

-- 3. 更新工人统计数据
UPDATE worker_stats 
SET completed_orders = completed_orders + 1,
    total_orders = total_orders + 1,
    total_earnings = total_earnings + ?,
    total_reviews = total_reviews + 1,
    average_rating = (average_rating * (total_reviews - 1) + ?) / total_reviews
WHERE worker_id = ?;

-- 4. 创建完成通知
INSERT INTO notifications (id, user_id, type, title, message, related_order_id)
VALUES (UUID(), ?, 'order_completed', '订单完成', '订单已完成，请评价', ?);

COMMIT;
```

### 6.2 分布式事务处理

#### Saga 模式实现
```rust
// 使用Saga模式处理跨服务的复杂业务流程
pub struct OrderCompletionSaga {
    order_service: Arc<OrderService>,
    payment_service: Arc<PaymentService>,
    notification_service: Arc<NotificationService>,
    audit_service: Arc<AuditService>,
}

impl OrderCompletionSaga {
    pub async fn execute(&self, order_id: String) -> Result<()> {
        let mut saga_log = SagaLog::new(order_id.clone());
        
        // Step 1: 完成订单
        match self.order_service.complete_order(&order_id).await {
            Ok(_) => saga_log.record_success("complete_order"),
            Err(e) => return self.rollback(saga_log, e).await,
        }
        
        // Step 2: 处理支付
        match self.payment_service.process_payment(&order_id).await {
            Ok(_) => saga_log.record_success("process_payment"),
            Err(e) => return self.rollback(saga_log, e).await,
        }
        
        // Step 3: 发送通知
        match self.notification_service.send_completion_notification(&order_id).await {
            Ok(_) => saga_log.record_success("send_notification"),
            Err(e) => {
                // 通知失败不需要回滚，记录错误即可
                tracing::warn!("Failed to send notification: {}", e);
            }
        }
        
        // Step 4: 记录审计日志
        self.audit_service.log_order_completion(&order_id).await;
        
        Ok(())
    }
    
    async fn rollback(&self, saga_log: SagaLog, error: Error) -> Result<()> {
        for step in saga_log.completed_steps.iter().rev() {
            match step.as_str() {
                "process_payment" => {
                    self.payment_service.refund_payment(&saga_log.order_id).await?;
                }
                "complete_order" => {
                    self.order_service.revert_completion(&saga_log.order_id).await?;
                }
                _ => {}
            }
        }
        
        Err(error)
    }
}
```

### 6.3 并发控制策略

#### 乐观锁实现
```rust
// 使用版本号实现乐观锁
#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub status: OrderStatus,
    pub version: i64,  // 版本号字段
    // ... 其他字段
}

impl OrderService {
    pub async fn update_order_status(
        &self,
        order_id: &str,
        new_status: OrderStatus,
        expected_version: i64,
    ) -> Result<Order> {
        let result = sqlx::query_as::<_, Order>(
            "UPDATE orders 
             SET status = ?, version = version + 1, updated_at = NOW()
             WHERE id = ? AND version = ?
             RETURNING *"
        )
        .bind(&new_status)
        .bind(order_id)
        .bind(expected_version)
        .fetch_optional(&self.db_pool)
        .await?;
        
        match result {
            Some(order) => Ok(order),
            None => Err(DomainError::OptimisticLockFailure(
                format!("Order {} was modified by another transaction", order_id)
            ))
        }
    }
}
```

#### 分布式锁实现
```rust
// 使用Redis实现分布式锁
pub struct DistributedLock {
    redis: Arc<redis::Client>,
    key: String,
    value: String,
    ttl: Duration,
}

impl DistributedLock {
    pub async fn acquire(&self) -> Result<bool> {
        let script = r#"
            if redis.call("SET", KEYS[1], ARGV[1], "PX", ARGV[2], "NX") then
                return 1
            else
                return 0
            end
        "#;
        
        let result: i32 = redis::cmd("EVAL")
            .arg(script)
            .arg(1)
            .arg(&self.key)
            .arg(&self.value)
            .arg(self.ttl.as_millis())
            .query_async(&mut self.redis.get_async_connection().await?)
            .await?;
            
        Ok(result == 1)
    }
    
    pub async fn release(&self) -> Result<()> {
        let script = r#"
            if redis.call("GET", KEYS[1]) == ARGV[1] then
                return redis.call("DEL", KEYS[1])
            else
                return 0
            end
        "#;
        
        redis::cmd("EVAL")
            .arg(script)
            .arg(1)
            .arg(&self.key)
            .arg(&self.value)
            .query_async(&mut self.redis.get_async_connection().await?)
            .await?;
            
        Ok(())
    }
}

// 使用分布式锁保护关键操作
pub async fn assign_order_to_worker(
    order_id: &str,
    worker_id: &str,
) -> Result<()> {
    let lock_key = format!("lock:order_assign:{}", order_id);
    let lock = DistributedLock::new(&lock_key, Duration::seconds(30));
    
    if !lock.acquire().await? {
        return Err(DomainError::ResourceLocked);
    }
    
    let result = async {
        // 执行订单分配逻辑
        assign_order_logic(order_id, worker_id).await
    }.await;
    
    lock.release().await?;
    result
}
```

## 7. 未来扩展需要的表结构

### 7.1 支付相关表

#### 支付记录表 (payments)
```sql
CREATE TABLE IF NOT EXISTS payments (
    id CHAR(36) NOT NULL COMMENT 'Payment ID',
    order_id CHAR(36) NOT NULL COMMENT 'Related order',
    payer_id CHAR(36) NOT NULL COMMENT 'User who made the payment',
    payee_id CHAR(36) NOT NULL COMMENT 'User who receives the payment',
    
    -- 支付信息
    amount DECIMAL(10,2) NOT NULL COMMENT 'Payment amount',
    currency CHAR(3) NOT NULL DEFAULT 'CNY' COMMENT 'Currency code',
    payment_method ENUM('wechat_pay', 'alipay', 'bank_card', 'balance') NOT NULL,
    
    -- 第三方支付信息
    external_transaction_id VARCHAR(100) NULL COMMENT 'External payment transaction ID',
    external_payment_data JSON NULL COMMENT 'External payment provider data',
    
    -- 状态
    status ENUM('pending', 'processing', 'completed', 'failed', 'refunded') NOT NULL DEFAULT 'pending',
    
    -- 时间戳
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP NULL,
    failed_at TIMESTAMP NULL,
    
    -- 备注
    failure_reason TEXT NULL,
    notes TEXT NULL,
    
    PRIMARY KEY (id),
    CONSTRAINT fk_payments_order_id 
        FOREIGN KEY (order_id) REFERENCES orders(id) 
        ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT fk_payments_payer_id 
        FOREIGN KEY (payer_id) REFERENCES users(id) 
        ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT fk_payments_payee_id 
        FOREIGN KEY (payee_id) REFERENCES users(id) 
        ON DELETE RESTRICT ON UPDATE CASCADE,
        
    UNIQUE KEY uk_external_transaction (external_transaction_id),
    INDEX idx_order_id (order_id),
    INDEX idx_status (status),
    INDEX idx_created_at (created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT='Payment transactions';
```

### 7.2 优惠券和营销表

#### 优惠券表 (coupons)
```sql
CREATE TABLE IF NOT EXISTS coupons (
    id CHAR(36) NOT NULL COMMENT 'Coupon ID',
    code VARCHAR(20) NOT NULL COMMENT 'Coupon code',
    name VARCHAR(100) NOT NULL COMMENT 'Coupon name',
    description TEXT NULL COMMENT 'Coupon description',
    
    -- 折扣信息
    discount_type ENUM('fixed_amount', 'percentage') NOT NULL,
    discount_value DECIMAL(10,2) NOT NULL COMMENT 'Discount value',
    min_order_amount DECIMAL(10,2) NULL COMMENT 'Minimum order amount to use coupon',
    max_discount_amount DECIMAL(10,2) NULL COMMENT 'Maximum discount amount for percentage coupons',
    
    -- 使用限制
    usage_limit INT NULL COMMENT 'Total usage limit (NULL = unlimited)',
    usage_limit_per_user INT NULL COMMENT 'Usage limit per user',
    used_count INT NOT NULL DEFAULT 0 COMMENT 'Times this coupon has been used',
    
    -- 适用范围
    applicable_service_categories JSON NULL COMMENT 'Applicable service category IDs',
    applicable_user_types SET('customer', 'worker') NULL COMMENT 'Applicable user types',
    
    -- 有效期
    valid_from TIMESTAMP NOT NULL COMMENT 'Coupon valid from',
    valid_until TIMESTAMP NOT NULL COMMENT 'Coupon valid until',
    
    -- 状态
    is_active BOOLEAN NOT NULL DEFAULT TRUE COMMENT 'Whether coupon is active',
    
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    PRIMARY KEY (id),
    UNIQUE KEY uk_coupon_code (code),
    INDEX idx_valid_period (valid_from, valid_until),
    INDEX idx_is_active (is_active)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT='Marketing coupons';
```

### 7.3 内容管理表

#### 文章/帮助内容表 (content_articles)
```sql
CREATE TABLE IF NOT EXISTS content_articles (
    id CHAR(36) NOT NULL COMMENT 'Article ID',
    title VARCHAR(200) NOT NULL COMMENT 'Article title',
    content LONGTEXT NOT NULL COMMENT 'Article content (Markdown format)',
    excerpt TEXT NULL COMMENT 'Article excerpt/summary',
    
    -- 分类
    category ENUM('help', 'tutorial', 'policy', 'announcement', 'tips') NOT NULL,
    tags JSON NULL COMMENT 'Article tags',
    
    -- SEO
    meta_title VARCHAR(200) NULL COMMENT 'SEO meta title',
    meta_description TEXT NULL COMMENT 'SEO meta description',
    slug VARCHAR(200) NOT NULL COMMENT 'URL slug',
    
    -- 发布信息
    author_id CHAR(36) NULL COMMENT 'Author (admin user)',
    is_published BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether article is published',
    published_at TIMESTAMP NULL COMMENT 'Publication timestamp',
    
    -- 统计
    view_count INT NOT NULL DEFAULT 0 COMMENT 'View count',
    
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    PRIMARY KEY (id),
    UNIQUE KEY uk_article_slug (slug),
    INDEX idx_category (category),
    INDEX idx_is_published (is_published),
    INDEX idx_published_at (published_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT='Content management articles';
```

### 7.4 分析和报表表

#### 用户行为分析表 (user_analytics)
```sql
CREATE TABLE IF NOT EXISTS user_analytics (
    id CHAR(36) NOT NULL COMMENT 'Analytics record ID',
    user_id CHAR(36) NULL COMMENT 'User ID (NULL for anonymous)',
    session_id VARCHAR(100) NOT NULL COMMENT 'Session identifier',
    
    -- 事件信息
    event_type VARCHAR(50) NOT NULL COMMENT 'Event type (page_view, click, search, etc.)',
    event_category VARCHAR(50) NOT NULL COMMENT 'Event category',
    event_action VARCHAR(100) NOT NULL COMMENT 'Event action',
    event_label VARCHAR(200) NULL COMMENT 'Event label',
    event_value INT NULL COMMENT 'Event value',
    
    -- 页面信息
    page_url VARCHAR(500) NULL COMMENT 'Page URL',
    page_title VARCHAR(200) NULL COMMENT 'Page title',
    referrer_url VARCHAR(500) NULL COMMENT 'Referrer URL',
    
    -- 设备信息
    user_agent TEXT NULL COMMENT 'User agent string',
    ip_address VARCHAR(45) NULL COMMENT 'IP address',
    device_type ENUM('mobile', 'tablet', 'desktop') NULL COMMENT 'Device type',
    os VARCHAR(50) NULL COMMENT 'Operating system',
    browser VARCHAR(50) NULL COMMENT 'Browser name',
    
    -- 位置信息
    city VARCHAR(50) NULL COMMENT 'City',
    country VARCHAR(50) NULL COMMENT 'Country',
    
    -- 时间信息
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    PRIMARY KEY (id),
    INDEX idx_user_id (user_id),
    INDEX idx_session_id (session_id),
    INDEX idx_event_type (event_type),
    INDEX idx_created_at (created_at),
    INDEX idx_page_analysis (page_url(100), created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT='User behavior analytics';
```

## 8. 性能监控查询

### 8.1 慢查询监控
```sql
-- 启用慢查询日志
SET GLOBAL slow_query_log = 'ON';
SET GLOBAL long_query_time = 2; -- 超过2秒的查询
SET GLOBAL log_queries_not_using_indexes = 'ON';

-- 查看慢查询统计
SELECT 
    DIGEST_TEXT as query_pattern,
    COUNT_STAR as exec_count,
    AVG_TIMER_WAIT/1000000000000 as avg_exec_time_sec,
    MAX_TIMER_WAIT/1000000000000 as max_exec_time_sec,
    SUM_ROWS_EXAMINED/COUNT_STAR as avg_rows_examined
FROM performance_schema.events_statements_summary_by_digest 
WHERE AVG_TIMER_WAIT > 2000000000000 -- 超过2秒
ORDER BY AVG_TIMER_WAIT DESC
LIMIT 10;
```

### 8.2 索引使用情况监控
```sql
-- 检查未使用的索引
SELECT 
    object_schema,
    object_name,
    index_name,
    count_star,
    count_read,
    count_write
FROM performance_schema.table_io_waits_summary_by_index_usage 
WHERE index_name IS NOT NULL 
  AND count_star = 0 
  AND object_schema = 'renoveasy';

-- 检查表扫描情况
SELECT 
    object_schema,
    object_name,
    count_read,
    count_write,
    count_fetch,
    SUM_TIMER_FETCH/1000000000000 as total_fetch_time_sec
FROM performance_schema.table_io_waits_summary_by_table 
WHERE object_schema = 'renoveasy'
ORDER BY count_fetch DESC;
```

### 8.3 连接池监控
```sql
-- 监控连接池状态
SELECT 
    VARIABLE_NAME,
    VARIABLE_VALUE
FROM performance_schema.global_status 
WHERE VARIABLE_NAME IN (
    'Connections',
    'Max_used_connections', 
    'Threads_connected',
    'Threads_running',
    'Connection_errors_max_connections'
);
```

## 9. 数据库运维建议

### 9.1 备份策略
```bash
#!/bin/bash
# 自动备份脚本
DB_NAME="renoveasy"
BACKUP_DIR="/backup/mysql"
DATE=$(date +%Y%m%d_%H%M%S)

# 全量备份（每日凌晨）
mysqldump --single-transaction --routines --triggers \
  --master-data=2 --flush-logs \
  -u backup_user -p${BACKUP_PASSWORD} \
  ${DB_NAME} > ${BACKUP_DIR}/full_backup_${DATE}.sql

# 压缩备份文件
gzip ${BACKUP_DIR}/full_backup_${DATE}.sql

# 删除7天前的备份
find ${BACKUP_DIR} -name "full_backup_*.sql.gz" -mtime +7 -delete
```

### 9.2 性能调优参数
```sql
-- MySQL配置优化建议 (my.cnf)
[mysqld]
# InnoDB设置
innodb_buffer_pool_size = 2G          # 设置为服务器内存的70-80%
innodb_log_file_size = 256M
innodb_flush_log_at_trx_commit = 1     # 数据安全优先
innodb_flush_method = O_DIRECT

# 连接设置
max_connections = 200
max_connect_errors = 10000
connect_timeout = 10

# 查询缓存（MySQL 8.0已弃用）
query_cache_type = 0
query_cache_size = 0

# 慢查询日志
slow_query_log = 1
long_query_time = 2
log_queries_not_using_indexes = 1

# 字符集
character-set-server = utf8mb4
collation-server = utf8mb4_unicode_ci
```

### 9.3 监控告警
```yaml
# 使用Prometheus监控的告警规则示例
groups:
- name: mysql.rules
  rules:
  - alert: MySQLDown
    expr: mysql_up == 0
    for: 0m
    labels:
      severity: critical
    annotations:
      summary: MySQL instance is down
      
  - alert: MySQLSlowQueries
    expr: increase(mysql_global_status_slow_queries[1m]) > 10
    for: 2m
    labels:
      severity: warning
    annotations:
      summary: High number of slow queries detected
      
  - alert: MySQLConnectionsHigh  
    expr: mysql_global_status_threads_connected / mysql_global_variables_max_connections > 0.8
    for: 2m
    labels:
      severity: warning
    annotations:
      summary: MySQL connection usage is above 80%
```

## 10. 实施计划

### 阶段1：基础扩展（2-3周）
1. 创建用户档案和工人资质相关表
2. 实现基础的地理位置索引
3. 设置Redis缓存基础架构
4. 实现基本的监控查询

### 阶段2：核心业务（3-4周）  
1. 创建订单相关完整表结构
2. 实现事务边界和一致性控制
3. 设置复合索引和查询优化
4. 实现缓存策略和预热机制

### 阶段3：高级功能（2-3周）
1. 实现评价和消息系统
2. 添加通知和收藏功能
3. 实现分布式锁和并发控制
4. 设置性能监控和告警

### 阶段4：未来扩展（1-2周）
1. 准备支付和营销相关表
2. 实现分析和报表功能
3. 优化查询性能和缓存策略
4. 完善备份和运维流程

## 总结

这个数据库设计方案为RenovEasy平台提供了：

1. **完整的数据模型**：覆盖用户管理、订单流程、评价系统、消息通讯等核心业务
2. **高性能的查询优化**：针对地理位置搜索、实时数据查询等场景进行了索引优化
3. **强大的缓存策略**：使用Redis实现多层次缓存，支持高并发访问
4. **可靠的数据一致性**：通过事务边界设计和分布式锁确保数据一致性
5. **良好的扩展性**：为未来的支付、营销、内容管理等功能预留了扩展空间

该方案遵循了现代数据库设计的最佳实践，能够支持平台的快速发展和扩展需求。