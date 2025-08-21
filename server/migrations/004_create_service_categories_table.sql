-- Migration: 004_create_service_categories_table
-- Description: Create service categories table for organizing renovation services
-- Date: 2025-08-20

-- Create service_categories table for hierarchical service organization
CREATE TABLE IF NOT EXISTS service_categories (
    id INT AUTO_INCREMENT NOT NULL COMMENT 'Category ID',
    name VARCHAR(50) NOT NULL COMMENT 'Category name in Chinese',
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
        ON DELETE SET NULL ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Create indexes for performance optimization
CREATE INDEX idx_service_categories_parent_id ON service_categories(parent_id);
CREATE INDEX idx_service_categories_is_active ON service_categories(is_active);
CREATE INDEX idx_service_categories_sort_order ON service_categories(sort_order);

-- Add table comment
ALTER TABLE service_categories COMMENT = 'Hierarchical service categories for renovation services';

-- Add column comments
ALTER TABLE service_categories
    MODIFY COLUMN id INT AUTO_INCREMENT NOT NULL COMMENT 'Unique category identifier',
    MODIFY COLUMN name VARCHAR(50) NOT NULL COMMENT 'Service category name in Chinese',
    MODIFY COLUMN name_en VARCHAR(50) NOT NULL COMMENT 'Service category name in English for i18n',
    MODIFY COLUMN parent_id INT NULL COMMENT 'Parent category ID for building category hierarchy',
    MODIFY COLUMN icon_url VARCHAR(255) NULL COMMENT 'URL to category icon for mobile app display',
    MODIFY COLUMN description TEXT NULL COMMENT 'Detailed description of the service category',
    MODIFY COLUMN is_active BOOLEAN NOT NULL DEFAULT TRUE COMMENT 'Whether this category is available for selection',
    MODIFY COLUMN sort_order INT NOT NULL DEFAULT 0 COMMENT 'Order for displaying categories in UI',
    MODIFY COLUMN created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Category creation timestamp',
    MODIFY COLUMN updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Last modification timestamp';

-- Insert initial service categories
INSERT INTO service_categories (name, name_en, description, sort_order) VALUES
('室内装修', 'Interior Decoration', '室内装修和装饰服务，包括整体设计和施工', 1),
('水电维修', 'Plumbing & Electrical', '水管和电路的安装、维修和改造服务', 2),
('家具安装', 'Furniture Installation', '各类家具的组装、安装和定制服务', 3),
('墙面处理', 'Wall Treatment', '刷漆、贴壁纸、瓷砖等墙面装饰处理', 4),
('地板铺设', 'Flooring Installation', '木地板、瓷砖、地毯等地面材料铺设', 5),
('厨卫改造', 'Kitchen & Bathroom Renovation', '厨房和卫生间的专业改造服务', 6),
('门窗安装', 'Doors & Windows Installation', '门窗的安装、更换和维修服务', 7),
('空调安装', 'Air Conditioning Installation', '空调设备的安装、维修和保养', 8);

-- Insert sub-categories for Interior Decoration
INSERT INTO service_categories (name, name_en, parent_id, description, sort_order) VALUES
('全屋装修', 'Complete Home Renovation', 1, '整套房屋的全面装修改造', 1),
('局部装修', 'Partial Renovation', 1, '房屋局部区域的装修改造', 2),
('软装设计', 'Interior Design', 1, '家居软装饰品的搭配和设计', 3);

-- Insert sub-categories for Plumbing & Electrical
INSERT INTO service_categories (name, name_en, parent_id, description, sort_order) VALUES
('水管维修', 'Plumbing Repair', 2, '水管漏水、堵塞等问题的维修', 1),
('电路维修', 'Electrical Repair', 2, '电路故障、开关插座等电气问题维修', 2),
('热水器安装', 'Water Heater Installation', 2, '热水器的安装和维修服务', 3);