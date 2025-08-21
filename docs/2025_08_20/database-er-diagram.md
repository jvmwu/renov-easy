# RenovEasy 数据库 ER 图设计

## 实体关系图 (Entity Relationship Diagram)

### 核心实体概览

```mermaid
erDiagram
    %% 用户相关实体
    users ||--o{ user_profiles : has
    users ||--o{ worker_qualifications : has
    users ||--o{ worker_stats : has
    
    %% 订单相关实体
    service_categories ||--o{ orders : categorizes
    users ||--o{ orders : creates_as_customer
    users ||--o{ orders : assigned_as_worker
    orders ||--o{ order_images : contains
    orders ||--o{ order_bids : receives
    users ||--o{ order_bids : makes
    
    %% 交互相关实体
    orders ||--|| reviews : generates
    users ||--o{ reviews : writes
    users ||--o{ reviews : receives
    users ||--o{ messages : sends
    users ||--o{ messages : receives
    orders ||--o{ messages : relates_to
    
    %% 其他关系
    users ||--o{ favorites : creates
    users ||--o{ favorites : is_favorited
    users ||--o{ notifications : receives
    orders ||--o{ notifications : triggers
    
    %% 认证相关（现有系统）
    users ||--o{ refresh_tokens : owns
    users ||--o{ auth_audit_log : generates
```

### 详细表结构关系

#### 1. 用户管理模块
```mermaid
erDiagram
    users {
        char(36) id PK
        varchar(64) phone_hash UK
        varchar(10) country_code
        enum user_type "customer,worker"
        timestamp created_at
        timestamp updated_at
        timestamp last_login_at
        boolean is_verified
        boolean is_blocked
    }
    
    user_profiles {
        char(36) user_id PK,FK
        varchar(50) display_name
        varchar(255) avatar_url
        text bio
        varchar(100) email
        varchar(50) wechat_id
        varchar(50) preferred_city
        text address
        decimal latitude
        decimal longitude
        timestamp created_at
        timestamp updated_at
    }
    
    worker_qualifications {
        char(36) id PK
        char(36) worker_id FK
        varchar(100) license_number
        enum license_type
        date license_expiry
        boolean is_verified
        timestamp verified_at
        char(36) verified_by
        varchar(255) certificate_url
        timestamp created_at
        timestamp updated_at
    }
    
    worker_stats {
        char(36) worker_id PK,FK
        int total_orders
        int completed_orders
        int canceled_orders
        decimal average_rating
        int total_reviews
        decimal total_earnings
        decimal this_month_earnings
        int total_work_hours
        int response_time_avg
        timestamp last_updated
    }
    
    users ||--|| user_profiles : has
    users ||--o{ worker_qualifications : has_multiple
    users ||--|| worker_stats : has_if_worker
```

#### 2. 订单管理模块
```mermaid
erDiagram
    service_categories {
        int id PK
        varchar(50) name
        varchar(50) name_en
        int parent_id FK
        varchar(255) icon_url
        text description
        boolean is_active
        int sort_order
        timestamp created_at
        timestamp updated_at
    }
    
    orders {
        char(36) id PK
        varchar(20) order_number UK
        char(36) customer_id FK
        char(36) worker_id FK
        int service_category_id FK
        varchar(100) title
        text description
        text address
        decimal latitude
        decimal longitude
        varchar(50) city
        varchar(50) district
        decimal budget_min
        decimal budget_max
        decimal final_price
        date preferred_start_date
        int estimated_duration
        date actual_start_date
        date actual_end_date
        enum status
        enum priority
        timestamp created_at
        timestamp updated_at
        timestamp published_at
        timestamp assigned_at
        timestamp completed_at
        timestamp canceled_at
        enum cancellation_reason
        text cancellation_note
    }
    
    order_images {
        char(36) id PK
        char(36) order_id FK
        varchar(500) image_url
        enum image_type
        text caption
        char(36) uploaded_by FK
        int file_size
        varchar(50) mime_type
        timestamp created_at
    }
    
    order_bids {
        char(36) id PK
        char(36) order_id FK
        char(36) worker_id FK
        decimal bid_price
        int estimated_days
        text message
        enum status
        timestamp created_at
        timestamp updated_at
        timestamp responded_at
    }
    
    service_categories ||--o{ orders : categorizes
    service_categories ||--o{ service_categories : parent_child
    orders ||--o{ order_images : contains
    orders ||--o{ order_bids : receives
```

#### 3. 评价和交流模块
```mermaid
erDiagram
    reviews {
        char(36) id PK
        char(36) order_id FK,UK
        char(36) reviewer_id FK
        char(36) reviewee_id FK
        tinyint overall_rating
        tinyint quality_rating
        tinyint communication_rating
        tinyint punctuality_rating
        text review_text
        boolean is_public
        boolean is_verified
        timestamp created_at
        timestamp updated_at
    }
    
    messages {
        char(36) id PK
        char(36) conversation_id
        char(36) sender_id FK
        char(36) receiver_id FK
        char(36) order_id FK
        enum message_type
        text content
        varchar(500) attachment_url
        boolean is_read
        timestamp read_at
        timestamp created_at
    }
    
    favorites {
        char(36) id PK
        char(36) user_id FK
        char(36) favorited_user_id FK
        timestamp created_at
    }
    
    notifications {
        char(36) id PK
        char(36) user_id FK
        enum type
        varchar(100) title
        text message
        char(36) related_order_id FK
        char(36) related_user_id FK
        boolean is_read
        boolean is_sent
        timestamp created_at
        timestamp read_at
        timestamp sent_at
    }
    
    orders ||--|| reviews : generates
    orders ||--o{ messages : relates_to
    orders ||--o{ notifications : triggers
```

### 索引策略可视化

#### 高频查询路径分析
```mermaid
graph TD
    A[客户端请求] --> B{查询类型}
    
    B -->|地理位置搜索| C[城市 + 坐标范围]
    C --> D[idx_location复合索引]
    D --> E[返回附近工人/订单]
    
    B -->|订单状态查询| F[用户ID + 状态]
    F --> G[idx_user_orders复合索引]
    G --> H[返回用户订单列表]
    
    B -->|评分排序| I[评分 + 订单数]
    I --> J[idx_rating_performance复合索引]
    J --> K[返回热门工人列表]
    
    B -->|实时消息| L[会话ID + 时间]
    L --> M[idx_conversation_time复合索引]
    M --> N[返回消息记录]
```

### 缓存层设计图

```mermaid
graph TB
    subgraph "应用层"
        APP[RenovEasy App]
    end
    
    subgraph "缓存层"
        L1[L1: 应用内存缓存]
        L2[L2: Redis缓存]
        
        subgraph "Redis数据结构"
            STRING[用户会话/计数器]
            HASH[用户资料/统计数据]
            ZSET[排行榜/评分排序]
            GEO[地理位置数据]
            LIST[消息队列/通知]
            SET[在线用户/标签]
        end
    end
    
    subgraph "数据库层"
        MYSQL[(MySQL 8.0)]
        
        subgraph "分区表"
            P1[orders_2024]
            P2[orders_2025]
            P3[messages_hash_0]
            P4[messages_hash_1]
        end
    end
    
    APP --> L1
    L1 -->|Cache Miss| L2
    L2 -->|Cache Miss| MYSQL
    
    L2 --> STRING
    L2 --> HASH
    L2 --> ZSET
    L2 --> GEO
    L2 --> LIST
    L2 --> SET
    
    MYSQL --> P1
    MYSQL --> P2
    MYSQL --> P3
    MYSQL --> P4
```

### 数据流向图

#### 订单处理流程
```mermaid
sequenceDiagram
    participant C as Customer
    participant A as App
    participant R as Redis
    participant D as Database
    participant W as Worker
    
    Note over C,W: 订单创建和分配流程
    
    C->>A: 创建订单
    A->>D: BEGIN TRANSACTION
    A->>D: INSERT INTO orders
    A->>D: INSERT INTO order_images
    A->>D: COMMIT
    
    A->>R: GEOADD orders:location
    A->>R: SET order:cache:{id}
    
    W->>A: 搜索附近订单
    A->>R: GEORADIUS orders:location
    A->>R: MGET order:cache:{ids}
    A-->>W: 返回订单列表
    
    W->>A: 提交竞价
    A->>D: INSERT INTO order_bids
    A->>R: LPUSH order:bids:{order_id}
    
    C->>A: 接受竞价
    A->>D: BEGIN TRANSACTION
    A->>D: UPDATE orders SET worker_id
    A->>D: UPDATE order_bids SET status
    A->>D: INSERT INTO notifications
    A->>D: COMMIT
    
    A->>R: DEL order:cache:{id}
    A->>R: SET order:assigned:{id}
```

### 性能优化策略图

```mermaid
mindmap
  root)数据库性能优化(
    索引优化
      复合索引
        地理位置搜索
        用户订单查询
        评分排序
      覆盖索引
        减少回表查询
        提升查询速度
      部分索引
        条件索引
        函数索引
    
    查询优化
      分页优化
        游标分页
        延迟关联
      关联查询优化
        适当反范式
        预计算统计
      避免N+1查询
        批量查询
        数据预加载
    
    缓存策略
      多级缓存
        应用缓存
        Redis缓存
      缓存模式
        Cache Aside
        Write Through
        Write Behind
      缓存预热
        热点数据
        定期刷新
    
    数据库架构
      读写分离
        主从复制
        读负载均衡
      分库分表
        垂直拆分
        水平拆分
      分区表
        时间分区
        哈希分区
```

### 数据一致性保证

```mermaid
graph TD
    subgraph "强一致性场景"
        A[订单状态变更] --> B[数据库事务]
        C[支付处理] --> B
        D[工人分配] --> B
    end
    
    subgraph "最终一致性场景"
        E[统计数据更新] --> F[异步处理]
        G[搜索索引更新] --> F
        H[缓存同步] --> F
    end
    
    subgraph "事务处理机制"
        B --> I{事务类型}
        I -->|单机事务| J[MySQL ACID]
        I -->|分布式事务| K[Saga模式]
        
        K --> L[步骤1: 订单更新]
        L --> M[步骤2: 支付处理]
        M --> N[步骤3: 通知发送]
        N --> O{是否成功}
        O -->|失败| P[补偿操作]
        O -->|成功| Q[完成]
    end
    
    subgraph "并发控制"
        R[乐观锁] --> S[版本号控制]
        T[悲观锁] --> U[SELECT FOR UPDATE]
        V[分布式锁] --> W[Redis分布式锁]
    end
```

### 监控和运维架构

```mermaid
graph TB
    subgraph "监控层"
        M1[Prometheus] --> D1[MySQL Exporter]
        M1 --> D2[Redis Exporter]
        M1 --> D3[应用监控]
        
        A1[Grafana] --> M1
        A2[AlertManager] --> M1
    end
    
    subgraph "数据库层"
        DB[(MySQL)]
        CACHE[(Redis)]
        
        subgraph "监控指标"
            QPS[查询QPS]
            SLOW[慢查询]
            CONN[连接数]
            CACHE_HIT[缓存命中率]
        end
    end
    
    subgraph "告警系统"
        ALERT1[数据库宕机告警]
        ALERT2[慢查询告警]
        ALERT3[连接数告警]
        ALERT4[缓存命中率告警]
    end
    
    subgraph "自动化运维"
        BACKUP[自动备份]
        CLEANUP[日志清理]
        OPTIMIZE[索引优化建议]
        SCALE[自动扩容]
    end
    
    DB --> QPS
    DB --> SLOW
    DB --> CONN
    CACHE --> CACHE_HIT
    
    A2 --> ALERT1
    A2 --> ALERT2
    A2 --> ALERT3
    A2 --> ALERT4
    
    M1 --> BACKUP
    M1 --> CLEANUP
    M1 --> OPTIMIZE
    M1 --> SCALE
```

## 核心设计原则总结

### 1. 数据模型设计原则
- **领域驱动**: 表设计紧密贴合业务领域模型
- **范式平衡**: 在3NF基础上适度反范式化提升性能
- **扩展性**: 预留扩展字段和表结构，支持业务发展
- **一致性**: 通过外键和约束保证数据完整性

### 2. 性能优化原则
- **索引策略**: 基于实际查询模式设计复合索引
- **查询优化**: 避免全表扫描，优化关联查询
- **缓存设计**: 多级缓存架构，合理的TTL策略
- **分区分片**: 大表分区，热数据集中

### 3. 可靠性原则
- **事务边界**: 明确的事务边界设计
- **并发控制**: 乐观锁和分布式锁结合
- **故障恢复**: 完善的备份和恢复策略
- **监控告警**: 全方位的性能和健康监控

### 4. 安全性原则
- **数据加密**: 敏感数据哈希存储
- **访问控制**: 基于角色的数据访问控制
- **审计日志**: 完整的操作审计记录
- **注入防护**: 参数化查询防止SQL注入

这个ER图设计为RenovEasy平台提供了清晰的数据架构蓝图，确保了系统的可扩展性、高性能和高可用性。
