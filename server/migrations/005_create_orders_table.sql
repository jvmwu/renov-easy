-- Migration: 005_create_orders_table
-- Description: Create orders table for renovation service requests
-- Date: 2025-08-20

-- Create orders table for managing renovation service requests
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
    UNIQUE KEY uk_orders_order_number (order_number),
    
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
    CHECK (estimated_duration IS NULL OR estimated_duration > 0)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Create indexes for performance optimization
CREATE INDEX idx_orders_customer_id ON orders(customer_id);
CREATE INDEX idx_orders_worker_id ON orders(worker_id);
CREATE INDEX idx_orders_status ON orders(status);
CREATE INDEX idx_orders_service_category ON orders(service_category_id);
CREATE INDEX idx_orders_location ON orders(city, latitude, longitude);
CREATE INDEX idx_orders_budget_range ON orders(budget_min, budget_max);
CREATE INDEX idx_orders_created_at ON orders(created_at);
CREATE INDEX idx_orders_published_at ON orders(published_at);
CREATE INDEX idx_orders_priority_status ON orders(priority, status);

-- 复合索引用于地理位置搜索
CREATE INDEX idx_orders_geo_search ON orders(city, status, service_category_id);
CREATE INDEX idx_orders_worker_orders ON orders(worker_id, status, created_at);

-- Add table comment
ALTER TABLE orders COMMENT = 'Main orders table for renovation services';

-- Add column comments
ALTER TABLE orders
    MODIFY COLUMN id CHAR(36) NOT NULL COMMENT 'Unique order identifier (UUID v4)',
    MODIFY COLUMN order_number VARCHAR(20) NOT NULL COMMENT 'Human-readable order number (e.g., RD202508200001)',
    MODIFY COLUMN customer_id CHAR(36) NOT NULL COMMENT 'User ID of the customer who created the order',
    MODIFY COLUMN worker_id CHAR(36) NULL COMMENT 'User ID of the assigned worker (NULL if not assigned)',
    MODIFY COLUMN service_category_id INT NOT NULL COMMENT 'Reference to service_categories.id',
    MODIFY COLUMN title VARCHAR(100) NOT NULL COMMENT 'Brief title/summary of the renovation request',
    MODIFY COLUMN description TEXT NOT NULL COMMENT 'Detailed description of work requirements',
    MODIFY COLUMN address TEXT NOT NULL COMMENT 'Full address where service is needed',
    MODIFY COLUMN latitude DECIMAL(10,8) NOT NULL COMMENT 'GPS latitude for location-based worker matching',
    MODIFY COLUMN longitude DECIMAL(11,8) NOT NULL COMMENT 'GPS longitude for location-based worker matching',
    MODIFY COLUMN city VARCHAR(50) NOT NULL COMMENT 'City name for filtering and search',
    MODIFY COLUMN district VARCHAR(50) NULL COMMENT 'District or area name within the city',
    MODIFY COLUMN budget_min DECIMAL(10,2) NOT NULL COMMENT 'Minimum budget customer is willing to pay (CNY)',
    MODIFY COLUMN budget_max DECIMAL(10,2) NOT NULL COMMENT 'Maximum budget customer is willing to pay (CNY)',
    MODIFY COLUMN final_price DECIMAL(10,2) NULL COMMENT 'Final agreed price after negotiation (CNY)',
    MODIFY COLUMN preferred_start_date DATE NULL COMMENT 'When customer prefers work to start',
    MODIFY COLUMN estimated_duration INT NULL COMMENT 'Estimated completion time in days',
    MODIFY COLUMN actual_start_date DATE NULL COMMENT 'When work actually started',
    MODIFY COLUMN actual_end_date DATE NULL COMMENT 'When work was actually completed',
    MODIFY COLUMN status ENUM('draft', 'published', 'assigned', 'in_progress', 'completed', 'canceled', 'disputed') NOT NULL DEFAULT 'draft' COMMENT 'Current status of the order',
    MODIFY COLUMN priority ENUM('low', 'normal', 'high', 'urgent') NOT NULL DEFAULT 'normal' COMMENT 'Priority level of the order',
    MODIFY COLUMN created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When order was created',
    MODIFY COLUMN updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'When order was last modified',
    MODIFY COLUMN published_at TIMESTAMP NULL COMMENT 'When order was made visible to workers',
    MODIFY COLUMN assigned_at TIMESTAMP NULL COMMENT 'When a worker was assigned to this order',
    MODIFY COLUMN completed_at TIMESTAMP NULL COMMENT 'When order was marked as completed',
    MODIFY COLUMN canceled_at TIMESTAMP NULL COMMENT 'When order was canceled',
    MODIFY COLUMN cancellation_reason ENUM('customer_request', 'worker_unavailable', 'price_disagreement', 'other') NULL COMMENT 'Reason for order cancellation',
    MODIFY COLUMN cancellation_note TEXT NULL COMMENT 'Additional details about why order was canceled';