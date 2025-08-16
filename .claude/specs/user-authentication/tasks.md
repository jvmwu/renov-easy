# Implementation Plan - User Authentication

## Task Overview
实现 RenovEasy 用户鉴权系统的 Rust 后端，按照 Clean Architecture 原则构建可扩展、安全的认证服务。任务按照从基础到完整功能的顺序组织，确保每个任务独立可测试。

## Steering Document Compliance
- 遵循 `structure.md` 定义的 Cargo workspace 结构
- 使用 `tech.md` 指定的 Rust 技术栈（Tokio, Actix-web, SQLx）
- 按照 `server/` 目录下的模块组织代码

## Atomic Task Requirements
**每个任务满足以下标准：**
- **文件范围**: 涉及 1-3 个相关文件
- **时间预估**: 15-30 分钟可完成
- **单一目标**: 一个可测试的成果
- **明确文件**: 指定要创建/修改的确切文件
- **代理友好**: 清晰的输入/输出，最小化上下文切换

## Tasks

### 1. 项目初始化和基础设置

- [x] 1. 创建 Rust workspace 配置文件 server/Cargo.toml
  - 文件: server/Cargo.toml
  - 定义 workspace members: api, core, infrastructure
  - 配置共享依赖: tokio, serde, sqlx, uuid, chrono
  - 设置 Rust 2021 edition 和基本元数据
  - _Requirements: 技术栈设置_

- [x] 2. 创建 core crate 的 Cargo.toml 和入口文件
  - 文件: server/core/Cargo.toml, server/core/src/lib.rs
  - 配置 core 依赖: serde, uuid, chrono, thiserror
  - 创建模块声明: domain, services, repositories, errors
  - 导出公共接口
  - _Requirements: 4.1_

- [x] 3. 创建 infrastructure crate 的 Cargo.toml 和入口文件
  - 文件: server/infrastructure/Cargo.toml, server/infrastructure/src/lib.rs
  - 配置依赖: sqlx, redis, reqwest, tokio
  - 创建模块声明: database, sms, cache
  - 设置异步运行时
  - _Requirements: 4.1_

- [x] 4. 创建 api crate 的 Cargo.toml 和入口文件
  - 文件: server/api/Cargo.toml, server/api/src/main.rs
  - 配置依赖: actix-web, jsonwebtoken, env_logger
  - 创建基本 HTTP 服务器结构
  - 配置端口 8080
  - _Requirements: 4.1_

### 2. 领域模型实现

- [x] 5. 创建 User 实体模型
  - 文件: server/core/src/domain/entities/user.rs
  - 定义 User struct 包含 id, phone_hash, country_code, user_type
  - 实现 UserType enum (Customer, Worker)
  - 添加序列化/反序列化 traits
  - _Requirements: 1.1, 3.1_

- [x] 6. 创建 VerificationCode 实体模型
  - 文件: server/core/src/domain/entities/verification_code.rs
  - 定义 VerificationCode struct 包含 phone, code, expires_at, attempts
  - 实现验证码生成方法 (6位随机数)
  - 添加过期检查方法
  - _Requirements: 2.1_

- [x] 7. 创建 Token 实体模型
  - 文件: server/core/src/domain/entities/token.rs
  - 定义 RefreshToken 和 TokenPair structs
  - 定义 Claims struct for JWT payload
  - 添加 token 相关常量 (过期时间等)
  - _Requirements: 4.1_

- [x] 8. 创建领域错误类型
  - 文件: server/core/src/errors/domain_error.rs
  - 使用 thiserror 定义 AuthError, TokenError, ValidationError
  - 实现错误转换和消息格式化
  - 支持中英文错误消息
  - _Requirements: 5.2_

### 3. 数据库和迁移设置

- [x] 9. 创建数据库迁移脚本 - users 表
  - 文件: server/migrations/001_create_users_table.sql
  - 创建 users 表结构 (id, phone_hash, country_code, user_type 等)
  - 添加必要的索引 (phone_hash, user_type)
  - 设置字符集为 UTF8MB4
  - _Requirements: 1.1_

- [x] 10. 创建数据库迁移脚本 - tokens 和 audit 表
  - 文件: server/migrations/002_create_tokens_audit_tables.sql
  - 创建 refresh_tokens 表
  - 创建 auth_audit_log 表
  - 添加外键约束和索引
  - _Requirements: 4.1, 5.4_

- [x] 11. 实现数据库连接池配置
  - 文件: server/infrastructure/src/database/connection.rs
  - 使用 SQLx 创建 MySQL 连接池
  - 配置连接池大小和超时
  - 实现健康检查方法
  - _Requirements: 性能要求_

### 4. Repository 层实现

- [x] 12. 定义 UserRepository trait
  - 文件: server/core/src/repositories/user_repository.rs
  - 定义异步方法: find_by_phone, create, update, find_by_id
  - 使用 Result 类型处理错误
  - 添加文档注释
  - _Requirements: 1.1, 3.1_

- [x] 13. 实现 MySQL UserRepository
  - 文件: server/infrastructure/src/database/mysql/user_repository_impl.rs
  - 实现 UserRepository trait 的所有方法
  - 使用 SQLx 执行查询
  - 处理手机号哈希
  - _Requirements: 1.1, 安全要求_

- [x] 14. 定义和实现 TokenRepository
  - 文件: server/core/src/repositories/token_repository.rs
  - 文件: server/infrastructure/src/database/mysql/token_repository_impl.rs
  - 定义方法: save_refresh_token, find_refresh_token, revoke_token
  - 实现 MySQL 版本
  - _Requirements: 4.1_

### 5. 缓存和外部服务集成

- [x] 15. 实现 Redis 缓存连接
  - 文件: server/infrastructure/src/cache/redis_client.rs
  - 配置 Redis 连接池
  - 实现基本操作: set_with_expiry, get, delete
  - 添加连接重试逻辑
  - _Requirements: 2.3_

- [x] 16. 创建验证码缓存服务
  - 文件: server/infrastructure/src/cache/verification_cache.rs
  - 实现验证码存储 (5分钟过期)
  - 实现尝试次数记录
  - 实现验证码验证逻辑
  - _Requirements: 2.1, 2.2_

- [x] 17. 创建 SMS 服务接口和 Mock 实现
  - 文件: server/infrastructure/src/sms/sms_service.rs
  - 文件: server/infrastructure/src/sms/mock_sms.rs
  - 定义 SmsService trait
  - 创建开发环境的 Mock 实现 (控制台输出)
  - _Requirements: 1.1, 2.1_

### 6. 核心业务服务

- [ ] 18. 创建 TokenService
  - 文件: server/core/src/services/token_service.rs
  - 实现 JWT 生成 (access_token 15分钟, refresh_token 7天)
  - 实现 token 验证方法
  - 实现 refresh token 逻辑
  - _Requirements: 4.1, 4.2, 4.3_

- [ ] 19. 创建 VerificationService
  - 文件: server/core/src/services/verification_service.rs
  - 整合 SMS 服务和缓存服务
  - 实现发送验证码逻辑
  - 实现验证码校验逻辑
  - _Requirements: 2.1, 2.2_

- [ ] 20. 创建 AuthService - 发送验证码功能
  - 文件: server/core/src/services/auth_service.rs (part 1)
  - 实现 send_verification_code 方法
  - 集成手机号验证
  - 集成速率限制检查
  - _Requirements: 1.1, 5.1_

- [ ] 21. 创建 AuthService - 验证码校验逻辑
  - 文件: server/core/src/services/auth_service.rs (part 2)
  - 实现 verify_code 方法的验证码校验部分
  - 调用 VerificationService 验证
  - 返回验证结果
  - _Requirements: 2.2_

- [ ] 22. 创建 AuthService - 用户创建和获取逻辑
  - 文件: server/core/src/services/auth_service.rs (part 3)
  - 实现用户查找逻辑
  - 实现新用户创建逻辑
  - 处理用户状态检查
  - _Requirements: 1.1, 3.1_

- [ ] 23. 创建 AuthService - Token 生成集成
  - 文件: server/core/src/services/auth_service.rs (part 4)
  - 集成 TokenService 生成 token pair
  - 处理新用户的类型选择标记
  - 返回认证响应
  - _Requirements: 4.1_

- [ ] 24. 创建 AuthService - 用户类型选择功能
  - 文件: server/core/src/services/auth_service.rs (part 5)
  - 实现 select_user_type 方法
  - 更新用户类型
  - 验证权限
  - _Requirements: 3.1, 3.2, 3.3_

### 7. API 层和中间件

- [ ] 25. 创建速率限制中间件
  - 文件: server/api/src/middleware/rate_limiter.rs
  - 使用 Redis 记录请求次数
  - 实现每手机号每小时3次SMS限制
  - 返回 429 错误码当超限
  - _Requirements: 5.1, 5.2_

- [ ] 26. 创建 JWT 认证中间件
  - 文件: server/api/src/middleware/auth_middleware.rs
  - 从 Authorization header 提取 token
  - 验证 token 有效性
  - 注入用户上下文到请求
  - _Requirements: 4.1, 6.1_

- [ ] 27. 创建 CORS 和安全中间件
  - 文件: server/api/src/middleware/cors.rs
  - 文件: server/api/src/middleware/security.rs
  - 配置 CORS 允许移动端访问
  - 强制 HTTPS (除开发环境)
  - _Requirements: 5.5, 安全要求_

### 8. API 端点实现

- [ ] 28. 创建认证 DTO 模型
  - 文件: server/api/src/dto/auth_dto.rs
  - 定义 SendCodeRequest, VerifyCodeRequest, AuthResponse
  - 实现请求验证
  - 添加 Serialize/Deserialize traits
  - _Requirements: 1.1, 2.1_

- [ ] 29. 实现发送验证码端点
  - 文件: server/api/src/routes/auth/send_code.rs
  - 创建 POST /api/v1/auth/send-code
  - 验证请求数据
  - 调用 AuthService
  - _Requirements: 1.1_

- [ ] 30. 实现验证码验证端点
  - 文件: server/api/src/routes/auth/verify_code.rs
  - 创建 POST /api/v1/auth/verify-code
  - 处理新用户和老用户逻辑
  - 返回 token 和用户信息
  - _Requirements: 2.1, 2.2_

- [ ] 31. 实现用户类型选择端点
  - 文件: server/api/src/routes/auth/select_type.rs
  - 创建 POST /api/v1/auth/select-type
  - 需要认证中间件
  - 更新用户类型
  - _Requirements: 3.1, 3.2_

- [ ] 32. 实现 token 刷新端点
  - 文件: server/api/src/routes/auth/refresh.rs
  - 创建 POST /api/v1/auth/refresh
  - 验证 refresh token
  - 生成新 token pair
  - _Requirements: 4.2_

- [ ] 33. 实现登出端点
  - 文件: server/api/src/routes/auth/logout.rs
  - 创建 POST /api/v1/auth/logout
  - 撤销 tokens
  - 清理会话
  - _Requirements: 4.4_
  - 文件: server/api/src/routes/auth/refresh.rs
  - 文件: server/api/src/routes/auth/logout.rs
  - 创建 POST /api/v1/auth/refresh
  - 创建 POST /api/v1/auth/logout
  - _Requirements: 4.2, 4.4_

### 9. 错误处理和日志

- [ ] 34. 创建统一错误处理器
  - 文件: server/api/src/handlers/error_handler.rs
  - 捕获所有错误类型
  - 格式化错误响应
  - 支持中英文错误消息
  - _Requirements: 错误处理设计_

- [ ] 35. 实现审计日志服务
  - 文件: server/core/src/services/audit_service.rs
  - 记录认证尝试
  - 记录失败原因
  - 异步写入数据库
  - _Requirements: 5.4, 安全要求_

### 10. 配置和环境管理

- [ ] 36. 创建配置管理模块
  - 文件: server/api/src/config.rs
  - 从环境变量读取配置
  - 定义 Config struct
  - 验证必需配置项
  - _Requirements: 配置管理_

- [ ] 37. 创建开发环境配置文件
  - 文件: server/.env.development
  - 配置数据库连接
  - 配置 Redis 连接
  - 配置 JWT 密钥
  - _Requirements: 配置管理_

### 11. 测试实现

- [ ] 38. 创建 TokenService 单元测试
  - 文件: server/core/src/services/token_service.rs (tests module)
  - 测试 token 生成
  - 测试 token 验证
  - 测试过期处理
  - _Requirements: 4.1_

- [ ] 39. 创建 VerificationService 单元测试
  - 文件: server/core/src/services/verification_service.rs (tests module)
  - 测试验证码生成
  - 测试验证逻辑
  - 测试过期和重试
  - _Requirements: 2.1, 2.2_

- [ ] 40. 创建注册流程集成测试
  - 文件: server/api/tests/auth_register_test.rs
  - 测试新用户注册流程
  - 测试验证码发送和验证
  - 使用测试数据库
  - _Requirements: 1.1, 2.1, 3.1_

- [ ] 41. 创建登录流程集成测试
  - 文件: server/api/tests/auth_login_test.rs
  - 测试已有用户登录
  - 测试 token 生成
  - 验证用户类型返回
  - _Requirements: 1.1, 4.1_

- [ ] 42. 创建速率限制测试
  - 文件: server/api/tests/rate_limit_test.rs
  - 测试 SMS 限制
  - 测试验证码重试限制
  - 测试限制重置
  - _Requirements: 5.1, 5.2, 5.3_

### 12. 文档和部署准备

- [ ] 43. 创建项目 README 文档
  - 文件: server/README.md
  - 项目描述和架构说明
  - 安装和运行指南
  - 环境变量配置说明
  - _Requirements: 文档要求_

- [ ] 44. 创建 API 接口文档
  - 文件: server/API.md
  - 文档化所有端点
  - 提供请求/响应示例
  - 错误码说明
  - _Requirements: 文档要求_

- [ ] 45. 创建应用 Dockerfile
  - 文件: server/Dockerfile
  - 多阶段构建 Rust 应用
  - 优化镜像大小
  - 设置运行时环境
  - _Requirements: 部署准备_

- [ ] 46. 创建 Docker Compose 配置
  - 文件: server/docker-compose.yml
  - 配置应用服务
  - 配置 MySQL 和 Redis 服务
  - 设置网络和卷
  - _Requirements: 部署准备_