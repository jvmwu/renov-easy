-- Migration: 007_create_order_interaction_tables
-- Description: Create tables for order images, bids, and reviews
-- Date: 2025-08-20

-- Create order_images table for storing order-related photos
CREATE TABLE IF NOT EXISTS order_images (
    id CHAR(36) NOT NULL COMMENT 'Image ID',
    order_id CHAR(36) NOT NULL COMMENT 'Reference to orders.id',
    image_url VARCHAR(500) NOT NULL COMMENT 'Image storage URL',
    image_type ENUM('requirement', 'before', 'progress', 'after') NOT NULL COMMENT 'Type/purpose of the image',
    caption TEXT NULL COMMENT 'Optional image description or caption',
    uploaded_by CHAR(36) NOT NULL COMMENT 'User who uploaded the image',
    file_size INT NULL COMMENT 'File size in bytes',
    mime_type VARCHAR(50) NULL COMMENT 'MIME type of the image (e.g., image/jpeg)',
    
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    PRIMARY KEY (id),
    CONSTRAINT fk_order_images_order_id 
        FOREIGN KEY (order_id) REFERENCES orders(id) 
        ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT fk_order_images_uploaded_by 
        FOREIGN KEY (uploaded_by) REFERENCES users(id) 
        ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Create indexes for order_images
CREATE INDEX idx_order_images_order_id ON order_images(order_id);
CREATE INDEX idx_order_images_image_type ON order_images(image_type);
CREATE INDEX idx_order_images_uploaded_by ON order_images(uploaded_by);

-- Create order_bids table for worker bidding system
CREATE TABLE IF NOT EXISTS order_bids (
    id CHAR(36) NOT NULL COMMENT 'Bid ID',
    order_id CHAR(36) NOT NULL COMMENT 'Reference to orders.id',
    worker_id CHAR(36) NOT NULL COMMENT 'Bidding worker ID',
    
    -- 竞价信息
    bid_price DECIMAL(10,2) NOT NULL COMMENT 'Bid price in CNY',
    estimated_days INT NOT NULL COMMENT 'Estimated completion time in days',
    message TEXT NULL COMMENT 'Bid message/proposal from worker',
    
    -- 状态
    status ENUM('pending', 'accepted', 'rejected', 'withdrawn') NOT NULL DEFAULT 'pending',
    
    -- 时间戳
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    responded_at TIMESTAMP NULL COMMENT 'When customer responded to this bid',
    
    PRIMARY KEY (id),
    CONSTRAINT fk_order_bids_order_id 
        FOREIGN KEY (order_id) REFERENCES orders(id) 
        ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT fk_order_bids_worker_id 
        FOREIGN KEY (worker_id) REFERENCES users(id) 
        ON DELETE CASCADE ON UPDATE CASCADE,
        
    -- 一个工人对一个订单只能有一个竞价
    UNIQUE KEY uk_order_bids_order_worker (order_id, worker_id),
    
    -- 业务约束
    CHECK (bid_price > 0),
    CHECK (estimated_days > 0)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Create indexes for order_bids
CREATE INDEX idx_order_bids_order_id ON order_bids(order_id);
CREATE INDEX idx_order_bids_worker_id ON order_bids(worker_id);
CREATE INDEX idx_order_bids_status ON order_bids(status);
CREATE INDEX idx_order_bids_created_at ON order_bids(created_at);
CREATE INDEX idx_order_bids_bid_price ON order_bids(bid_price);

-- Create reviews table for order completion feedback
CREATE TABLE IF NOT EXISTS reviews (
    id CHAR(36) NOT NULL COMMENT 'Review ID',
    order_id CHAR(36) NOT NULL COMMENT 'Reference to completed order',
    reviewer_id CHAR(36) NOT NULL COMMENT 'User who wrote the review (customer)',
    reviewee_id CHAR(36) NOT NULL COMMENT 'User being reviewed (worker)',
    
    -- 评分 (1-5星)
    overall_rating TINYINT NOT NULL COMMENT 'Overall rating (1-5)',
    quality_rating TINYINT NOT NULL COMMENT 'Work quality rating (1-5)',
    communication_rating TINYINT NOT NULL COMMENT 'Communication rating (1-5)',
    punctuality_rating TINYINT NOT NULL COMMENT 'Punctuality rating (1-5)',
    
    -- 评价内容
    review_text TEXT NULL COMMENT 'Written review content',
    
    -- 状态
    is_public BOOLEAN NOT NULL DEFAULT TRUE COMMENT 'Whether review is visible to public',
    is_verified BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether review is verified by admin',
    
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
    UNIQUE KEY uk_reviews_order_review (order_id),
    
    -- 评分约束
    CHECK (overall_rating BETWEEN 1 AND 5),
    CHECK (quality_rating BETWEEN 1 AND 5),
    CHECK (communication_rating BETWEEN 1 AND 5),
    CHECK (punctuality_rating BETWEEN 1 AND 5)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Create indexes for reviews
CREATE INDEX idx_reviews_reviewee_id ON reviews(reviewee_id);
CREATE INDEX idx_reviews_overall_rating ON reviews(overall_rating);
CREATE INDEX idx_reviews_is_public ON reviews(is_public);
CREATE INDEX idx_reviews_created_at ON reviews(created_at);
CREATE INDEX idx_reviews_reviewer_id ON reviews(reviewer_id);

-- Add table comments
ALTER TABLE order_images COMMENT = 'Images associated with orders (requirement photos, progress updates, etc.)';
ALTER TABLE order_bids COMMENT = 'Worker bids for published orders';
ALTER TABLE reviews COMMENT = 'Customer reviews for completed orders';

-- Add column comments for order_images
ALTER TABLE order_images
    MODIFY COLUMN id CHAR(36) NOT NULL COMMENT 'Unique image identifier (UUID v4)',
    MODIFY COLUMN order_id CHAR(36) NOT NULL COMMENT 'Order this image belongs to',
    MODIFY COLUMN image_url VARCHAR(500) NOT NULL COMMENT 'Full URL to the stored image file',
    MODIFY COLUMN image_type ENUM('requirement', 'before', 'progress', 'after') NOT NULL COMMENT 'Purpose: requirement=initial photos, before/after=work photos, progress=update photos',
    MODIFY COLUMN caption TEXT NULL COMMENT 'Optional description or notes about the image',
    MODIFY COLUMN uploaded_by CHAR(36) NOT NULL COMMENT 'User ID of who uploaded this image (customer or worker)',
    MODIFY COLUMN file_size INT NULL COMMENT 'Image file size in bytes for storage management',
    MODIFY COLUMN mime_type VARCHAR(50) NULL COMMENT 'Image format (image/jpeg, image/png, etc.)',
    MODIFY COLUMN created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When image was uploaded';

-- Add column comments for order_bids  
ALTER TABLE order_bids
    MODIFY COLUMN id CHAR(36) NOT NULL COMMENT 'Unique bid identifier (UUID v4)',
    MODIFY COLUMN order_id CHAR(36) NOT NULL COMMENT 'Order being bid on',
    MODIFY COLUMN worker_id CHAR(36) NOT NULL COMMENT 'Worker making the bid',
    MODIFY COLUMN bid_price DECIMAL(10,2) NOT NULL COMMENT 'Price quoted by worker in Chinese Yuan',
    MODIFY COLUMN estimated_days INT NOT NULL COMMENT 'How many days worker estimates to complete the job',
    MODIFY COLUMN message TEXT NULL COMMENT 'Optional message from worker explaining their approach or qualifications',
    MODIFY COLUMN status ENUM('pending', 'accepted', 'rejected', 'withdrawn') NOT NULL DEFAULT 'pending' COMMENT 'Current status of the bid',
    MODIFY COLUMN created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When bid was submitted',
    MODIFY COLUMN updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'When bid was last modified',
    MODIFY COLUMN responded_at TIMESTAMP NULL COMMENT 'When customer accepted/rejected this bid';

-- Add column comments for reviews
ALTER TABLE reviews
    MODIFY COLUMN id CHAR(36) NOT NULL COMMENT 'Unique review identifier (UUID v4)',
    MODIFY COLUMN order_id CHAR(36) NOT NULL COMMENT 'Completed order being reviewed',
    MODIFY COLUMN reviewer_id CHAR(36) NOT NULL COMMENT 'Customer who wrote the review',
    MODIFY COLUMN reviewee_id CHAR(36) NOT NULL COMMENT 'Worker being reviewed',
    MODIFY COLUMN overall_rating TINYINT NOT NULL COMMENT 'Overall satisfaction rating from 1 (worst) to 5 (best)',
    MODIFY COLUMN quality_rating TINYINT NOT NULL COMMENT 'Work quality rating from 1 to 5',
    MODIFY COLUMN communication_rating TINYINT NOT NULL COMMENT 'Communication effectiveness rating from 1 to 5',
    MODIFY COLUMN punctuality_rating TINYINT NOT NULL COMMENT 'Timeliness and punctuality rating from 1 to 5',
    MODIFY COLUMN review_text TEXT NULL COMMENT 'Written review content with detailed feedback',
    MODIFY COLUMN is_public BOOLEAN NOT NULL DEFAULT TRUE COMMENT 'Whether this review is visible to other users',
    MODIFY COLUMN is_verified BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether admin has verified this review is genuine',
    MODIFY COLUMN created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When review was submitted',
    MODIFY COLUMN updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'When review was last modified';