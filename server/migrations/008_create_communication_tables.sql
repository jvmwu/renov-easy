-- Migration: 008_create_communication_tables
-- Description: Create tables for messaging, notifications, and favorites
-- Date: 2025-08-20

-- Create messages table for customer-worker communication
CREATE TABLE IF NOT EXISTS messages (
    id CHAR(36) NOT NULL COMMENT 'Message ID',
    conversation_id CHAR(36) NOT NULL COMMENT 'Conversation/chat ID for grouping messages',
    sender_id CHAR(36) NOT NULL COMMENT 'User who sent the message',
    receiver_id CHAR(36) NOT NULL COMMENT 'User who receives the message',
    order_id CHAR(36) NULL COMMENT 'Related order (if message is order-specific)',
    
    -- 消息内容
    message_type ENUM('text', 'image', 'location', 'system') NOT NULL DEFAULT 'text' COMMENT 'Type of message content',
    content TEXT NOT NULL COMMENT 'Message content (text, image URL, coordinates, etc.)',
    attachment_url VARCHAR(500) NULL COMMENT 'URL to attached file (images, documents)',
    
    -- 状态
    is_read BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether message has been read by receiver',
    read_at TIMESTAMP NULL COMMENT 'When message was marked as read',
    
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
        ON DELETE SET NULL ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Create indexes for messages
CREATE INDEX idx_messages_conversation_id ON messages(conversation_id);
CREATE INDEX idx_messages_sender_id ON messages(sender_id);
CREATE INDEX idx_messages_receiver_id ON messages(receiver_id);
CREATE INDEX idx_messages_order_id ON messages(order_id);
CREATE INDEX idx_messages_created_at ON messages(created_at);
CREATE INDEX idx_messages_is_read ON messages(is_read);

-- 复合索引用于会话查询
CREATE INDEX idx_messages_conversation_time ON messages(conversation_id, created_at DESC);
CREATE INDEX idx_messages_receiver_unread ON messages(receiver_id, is_read, created_at DESC);

-- Create favorites table for customers to favorite workers
CREATE TABLE IF NOT EXISTS favorites (
    id CHAR(36) NOT NULL COMMENT 'Favorite record ID',
    user_id CHAR(36) NOT NULL COMMENT 'User who added the favorite (typically customer)',
    favorited_user_id CHAR(36) NOT NULL COMMENT 'User being favorited (typically worker)',
    
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    PRIMARY KEY (id),
    CONSTRAINT fk_favorites_user_id 
        FOREIGN KEY (user_id) REFERENCES users(id) 
        ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT fk_favorites_favorited_user_id 
        FOREIGN KEY (favorited_user_id) REFERENCES users(id) 
        ON DELETE CASCADE ON UPDATE CASCADE,
        
    -- 防止重复收藏
    UNIQUE KEY uk_favorites_user_favorite (user_id, favorited_user_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Create indexes for favorites
CREATE INDEX idx_favorites_user_id ON favorites(user_id);
CREATE INDEX idx_favorites_favorited_user_id ON favorites(favorited_user_id);

-- Create notifications table for system notifications
CREATE TABLE IF NOT EXISTS notifications (
    id CHAR(36) NOT NULL COMMENT 'Notification ID',
    user_id CHAR(36) NOT NULL COMMENT 'User who receives this notification',
    
    -- 通知内容
    type ENUM('order_assigned', 'new_bid', 'bid_accepted', 'bid_rejected', 'order_completed', 'order_canceled', 'review_received', 'message_received', 'system_announcement') 
        NOT NULL COMMENT 'Type of notification',
    title VARCHAR(100) NOT NULL COMMENT 'Notification title/subject',
    message TEXT NOT NULL COMMENT 'Notification content/body',
    
    -- 关联数据
    related_order_id CHAR(36) NULL COMMENT 'Related order ID (if applicable)',
    related_user_id CHAR(36) NULL COMMENT 'Related user ID (e.g., who sent message, who made bid)',
    
    -- 状态
    is_read BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether user has read this notification',
    is_sent BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether push notification was sent to device',
    
    -- 时间戳
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    read_at TIMESTAMP NULL COMMENT 'When user marked notification as read',
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
        ON DELETE SET NULL ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Create indexes for notifications
CREATE INDEX idx_notifications_user_id ON notifications(user_id);
CREATE INDEX idx_notifications_type ON notifications(type);
CREATE INDEX idx_notifications_is_read ON notifications(is_read);
CREATE INDEX idx_notifications_created_at ON notifications(created_at);
CREATE INDEX idx_notifications_user_unread ON notifications(user_id, is_read, created_at DESC);

-- Add table comments
ALTER TABLE messages COMMENT = 'Messages between users (customers and workers)';
ALTER TABLE favorites COMMENT = 'User favorites (customers favoriting workers)';
ALTER TABLE notifications COMMENT = 'System notifications for users';

-- Add column comments for messages
ALTER TABLE messages
    MODIFY COLUMN id CHAR(36) NOT NULL COMMENT 'Unique message identifier (UUID v4)',
    MODIFY COLUMN conversation_id CHAR(36) NOT NULL COMMENT 'Groups related messages together (e.g., all messages between two users)',
    MODIFY COLUMN sender_id CHAR(36) NOT NULL COMMENT 'User who sent this message',
    MODIFY COLUMN receiver_id CHAR(36) NOT NULL COMMENT 'User who should receive this message',
    MODIFY COLUMN order_id CHAR(36) NULL COMMENT 'Order this message relates to (NULL for general conversation)',
    MODIFY COLUMN message_type ENUM('text', 'image', 'location', 'system') NOT NULL DEFAULT 'text' COMMENT 'Type of message: text=normal message, image=photo, location=GPS coordinates, system=automated',
    MODIFY COLUMN content TEXT NOT NULL COMMENT 'Main message content (text, image description, coordinates as JSON, etc.)',
    MODIFY COLUMN attachment_url VARCHAR(500) NULL COMMENT 'URL to attached file when message_type is image',
    MODIFY COLUMN is_read BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether recipient has seen this message',
    MODIFY COLUMN read_at TIMESTAMP NULL COMMENT 'When recipient marked message as read',
    MODIFY COLUMN created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When message was sent';

-- Add column comments for favorites
ALTER TABLE favorites
    MODIFY COLUMN id CHAR(36) NOT NULL COMMENT 'Unique favorite record identifier (UUID v4)',
    MODIFY COLUMN user_id CHAR(36) NOT NULL COMMENT 'User who added this favorite (usually a customer)',
    MODIFY COLUMN favorited_user_id CHAR(36) NOT NULL COMMENT 'User who was favorited (usually a worker)',
    MODIFY COLUMN created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When favorite was added';

-- Add column comments for notifications
ALTER TABLE notifications
    MODIFY COLUMN id CHAR(36) NOT NULL COMMENT 'Unique notification identifier (UUID v4)',
    MODIFY COLUMN user_id CHAR(36) NOT NULL COMMENT 'User who should receive this notification',
    MODIFY COLUMN type ENUM('order_assigned', 'new_bid', 'bid_accepted', 'bid_rejected', 'order_completed', 'order_canceled', 'review_received', 'message_received', 'system_announcement') NOT NULL COMMENT 'Category of notification for proper handling and display',
    MODIFY COLUMN title VARCHAR(100) NOT NULL COMMENT 'Short notification title for display in notification list',
    MODIFY COLUMN message TEXT NOT NULL COMMENT 'Detailed notification content with relevant information',
    MODIFY COLUMN related_order_id CHAR(36) NULL COMMENT 'Order related to this notification (for order_assigned, new_bid, etc.)',
    MODIFY COLUMN related_user_id CHAR(36) NULL COMMENT 'Other user involved in this notification (message sender, bid maker, etc.)',
    MODIFY COLUMN is_read BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether user has viewed this notification',
    MODIFY COLUMN is_sent BOOLEAN NOT NULL DEFAULT FALSE COMMENT 'Whether push notification was successfully sent to user device',
    MODIFY COLUMN created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'When notification was created',
    MODIFY COLUMN read_at TIMESTAMP NULL COMMENT 'When user opened/read the notification',
    MODIFY COLUMN sent_at TIMESTAMP NULL COMMENT 'When push notification was delivered to device';