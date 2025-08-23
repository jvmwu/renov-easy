# Implementation Plan - 无密码认证系统 (Passwordless Authentication System)

## Task Overview

本实施计划将无密码认证系统分解为原子任务，每个任务专注于单一功能，可在15-30分钟内完成。任务设计充分利用现有代码基础，遵循项目结构规范，确保可独立实施和测试。

## Steering Document Compliance

任务分解严格遵循 structure.md 中的项目组织规范和 tech.md 中的技术标准：
- 文件组织按照 server/api, server/core, server/infra 模块划分
- 命名规范遵循 snake_case 文件名，PascalCase 类型
- 最大化重用现有组件，减少新代码开发

## Atomic Task Requirements

每个任务满足以下原子性要求：
- **文件范围**: 涉及1-3个相关文件
- **时间限制**: 15-30分钟可完成
- **单一目标**: 一个可测试的输出
- **具体文件**: 明确指定要创建/修改的文件
- **清晰接口**: 输入输出明确，最小化上下文切换

## Tasks

### Phase 1: 数据库和基础设施准备

- [x] 1. 创建 verification_codes 表的数据库迁移脚本
  - File: server/migrations/002_create_verification_codes_table.sql
  - 创建表结构：id, phone_hash, code_encrypted, attempts, max_attempts, expires_at, created_at, is_used, is_locked
  - 添加索引：phone_hash, expires_at, is_used
  - 添加元数据字段：ip_address, user_agent
  - Purpose: 为OTP验证码提供加密持久化存储
  - _Leverage: server/migrations/001_create_users_table.sql 结构参考_
  - _Requirements: 2.1, 4.1, 8.1_

- [x] 2. 创建 refresh_tokens 表的数据库迁移脚本
  - File: server/migrations/003_create_refresh_tokens_table.sql
  - 创建表结构：id, user_id, token_hash, created_at, expires_at, last_used_at, is_revoked
  - 添加设备信息字段：device_info, ip_address
  - 添加索引：user_id, expires_at, is_revoked, last_used_at
  - Purpose: 为JWT刷新令牌提供安全存储
  - _Leverage: server/migrations/001_create_users_table.sql 结构参考_
  - _Requirements: 5.2, 5.5_

- [x] 3. 创建 auth_audit_log 表的数据库迁移脚本
  - File: server/migrations/004_create_auth_audit_log_table.sql
  - 创建表结构：id, user_id, phone_hash, action, success, ip_address, user_agent
  - 添加错误跟踪字段：error_code, error_message
  - 添加metadata JSON字段和created_at时间戳
  - 添加索引：user_id, phone_hash, action, created_at, success, ip_address
  - Purpose: 为审计日志提供持久化存储
  - _Leverage: server/migrations/001_create_users_table.sql 结构参考_
  - _Requirements: 7.1, 7.8_

### Phase 2: 核心加密和工具实现

- [x] 4. 实现 OTP 加密服务模块
  - File: server/core/src/services/crypto/otp_encryption.rs
  - 实现 encrypt_otp() 和 decrypt_otp() 使用 AES-256-GCM
  - 添加密钥管理函数 get_encryption_key()
  - Purpose: 提供OTP安全加密存储
  - _Leverage: 使用 aes-gcm crate_
  - _Requirements: 8.1, 8.2_

- [x] 5. 增强电话号码验证工具
  - File: server/core/src/services/auth/phone_utils.rs (修改现有)
  - 添加中国手机号验证 validate_chinese_phone()
  - 添加澳大利亚手机号验证 validate_australian_phone()
  - Purpose: 支持目标市场的手机号格式
  - _Leverage: 现有 is_valid_phone_format() 函数_
  - _Requirements: 1.1, 1.6, 1.7_

- [x] 6. 实现速率限制服务
  - File: server/infra/src/cache/rate_limiter.rs
  - 实现 RateLimiterTrait 接口
  - 添加 check_phone_limit() 和 check_ip_limit() 方法
  - Purpose: 防止滥用和DDoS攻击
  - _Leverage: server/core/src/services/auth/rate_limiter.rs trait_
  - _Requirements: 3.1, 3.3, 3.8_

### Phase 3: SMS服务集成

- [ ] 7. 实现 Twilio SMS 服务提供商
  - File: server/infra/src/sms/twilio_service.rs
  - 实现 SmsServiceTrait 接口
  - 添加 send_sms() 方法使用 Twilio API
  - Purpose: 集成主要SMS服务提供商
  - _Leverage: server/core/src/services/verification/traits.rs_
  - _Requirements: 2.9_

- [ ] 8. 实现 AWS SNS SMS 服务提供商
  - File: server/infra/src/sms/aws_sns_service.rs
  - 实现 SmsServiceTrait 接口
  - 添加 send_sms() 方法使用 AWS SNS API
  - Purpose: 集成备用SMS服务提供商
  - _Leverage: server/core/src/services/verification/traits.rs_
  - _Requirements: 2.10_

- [ ] 9. 实现 SMS 服务故障转移管理器
  - File: server/infra/src/sms/failover_manager.rs
  - 创建 SmsFailoverManager 结构体
  - 实现自动故障检测和切换逻辑
  - Purpose: 确保SMS服务高可用性
  - _Leverage: twilio_service.rs, aws_sns_service.rs_
  - _Requirements: 2.10_

### Phase 4: 验证码管理服务

- [ ] 10. 增强 VerificationService 的 OTP 生成逻辑
  - File: server/core/src/services/verification/service.rs (修改现有)
  - 更新 generate_code() 使用 CSPRNG
  - 添加验证码格式验证 is_valid_otp_format()
  - Purpose: 生成密码学安全的验证码
  - _Leverage: 现有 VerificationService 结构_
  - _Requirements: 2.1_

- [ ] 11. 实现验证码 Redis 存储逻辑
  - File: server/infra/src/cache/verification_cache.rs (修改现有)
  - 更新 store_code() 使用加密存储
  - 添加 TTL 设置（5分钟过期）
  - Purpose: 安全存储验证码到Redis
  - _Leverage: 现有 verification_cache.rs, otp_encryption.rs_
  - _Requirements: 2.11, 4.7, 8.2_

- [ ] 12. 实现验证码验证和尝试管理
  - File: server/core/src/services/verification/validator.rs
  - 创建 OtpValidator 结构体
  - 实现 verify_with_attempts() 方法，处理尝试计数
  - Purpose: 管理验证尝试和账户锁定
  - _Leverage: verification_cache.rs_
  - _Requirements: 2.3, 2.4, 6.2_

### Phase 5: 认证服务核心逻辑

- [ ] 13. 实现发送验证码 API 处理器
  - File: server/api/src/routes/auth/send_code.rs (修改现有)
  - 完善 send_verification_code 处理函数
  - 添加请求验证和错误处理
  - Purpose: 处理发送验证码的HTTP请求
  - _Leverage: 现有路由结构, DTOValidation_
  - _Requirements: 1.3_

- [ ] 14. 实现验证码校验 API 处理器
  - File: server/api/src/routes/auth/verify_code.rs (修改现有)
  - 完善 verify_code 处理函数
  - 添加令牌生成逻辑
  - Purpose: 处理验证码验证和登录
  - _Leverage: 现有路由结构, TokenService_
  - _Requirements: 2.1, 5.1, 5.2_

- [ ] 15. 增强 AuthService 的速率限制集成
  - File: server/core/src/services/auth/service.rs (修改现有)
  - 在 send_verification_code() 中集成速率限制检查
  - 添加速率限制错误处理
  - Purpose: 在认证服务中强制执行速率限制
  - _Leverage: 现有 AuthService, rate_limiter.rs_
  - _Requirements: 3.1, 3.2_

### Phase 6: JWT令牌管理

- [ ] 16. 配置 JWT RS256 密钥对
  - File: server/core/src/services/token/keys.rs
  - 创建 RSA 密钥管理模块
  - 实现密钥加载和轮换逻辑
  - Purpose: 管理JWT签名密钥
  - _Leverage: jsonwebtoken crate_
  - _Requirements: 5.3_

- [ ] 17. 增强 TokenService 令牌生成
  - File: server/core/src/services/token/service.rs (修改现有)
  - 更新 generate_tokens() 使用 RS256 算法
  - 添加必要的 claims（用户ID、手机号哈希）
  - Purpose: 生成安全的JWT令牌
  - _Leverage: 现有 TokenService 结构_
  - _Requirements: 5.3, 5.4_

- [ ] 18. 实现刷新令牌存储逻辑
  - File: server/infra/src/database/mysql/token_repository_impl.rs (修改现有)
  - 实现 store_refresh_token() 方法
  - 添加令牌哈希存储逻辑
  - Purpose: 持久化刷新令牌
  - _Leverage: 现有 TokenRepository trait_
  - _Requirements: 5.5_

### Phase 7: 审计日志实现

- [ ] 19. 增强审计日志数据模型
  - File: server/core/src/domain/entities/audit.rs (修改现有)
  - 添加认证特定的事件类型枚举
  - 实现手机号脱敏函数 mask_phone_for_audit()
  - Purpose: 定义审计日志数据结构
  - _Leverage: 现有 AuditLog 结构_
  - _Requirements: 7.7_

- [ ] 20. 实现审计日志仓储
  - File: server/infra/src/database/mysql/audit_repository_impl.rs
  - 实现 AuditLogRepository trait
  - 添加 create_auth_event() 方法
  - Purpose: 持久化审计日志到数据库
  - _Leverage: server/core/src/repositories/audit/trait.rs_
  - _Requirements: 7.1, 7.8_

- [ ] 21. 集成审计日志到认证流程
  - File: server/core/src/services/auth/service.rs (修改现有)
  - 在关键操作点添加审计日志调用
  - 记录成功/失败的认证尝试
  - Purpose: 确保所有认证事件被记录
  - _Leverage: AuditService, 现有 AuthService_
  - _Requirements: 7.2, 7.3, 7.4_

### Phase 8: 防暴力破解机制

- [ ] 22. 实现账户锁定服务
  - File: server/core/src/services/auth/account_lock.rs
  - 创建 AccountLockService 结构体
  - 实现 lock_account() 和 is_locked() 方法
  - Purpose: 管理暴力破解后的账户锁定
  - _Leverage: Redis缓存_
  - _Requirements: 6.2, 6.3_

- [ ] 23. 实现延迟响应机制
  - File: server/core/src/services/auth/delay_response.rs
  - 创建渐进式延迟算法
  - 实现 calculate_delay() 基于失败次数
  - Purpose: 通过延迟响应减缓暴力破解
  - _Leverage: 速率限制服务_
  - _Requirements: 6.1_

- [ ] 24. 添加分布式攻击检测
  - File: server/core/src/services/auth/attack_detector.rs
  - 创建 AttackDetector 服务
  - 实现 IP 范围异常检测逻辑
  - Purpose: 识别和阻止分布式攻击
  - _Leverage: 审计日志数据_
  - _Requirements: 6.4, 6.5_

### Phase 9: 国际化和错误处理

- [ ] 25. 创建认证错误消息的中文翻译
  - File: server/api/src/i18n/locales/zh-CN/auth.toml
  - 添加所有认证相关错误消息的中文翻译
  - 包含速率限制、验证码错误等消息
  - Purpose: 支持中文用户界面
  - _Leverage: 现有 i18n 系统_
  - _Requirements: 1.2_

- [ ] 26. 创建认证错误消息的英文翻译
  - File: server/api/src/i18n/locales/en-US/auth.toml
  - 添加所有认证相关错误消息的英文版本
  - 确保消息清晰可操作
  - Purpose: 支持英文用户界面
  - _Leverage: 现有 i18n 系统_
  - _Requirements: 1.2_

- [ ] 27. 实现错误响应标准化
  - File: server/api/src/handlers/error.rs (修改现有)
  - 添加认证特定错误处理
  - 实现恒定时间响应逻辑
  - Purpose: 统一错误处理和防止时序攻击
  - _Leverage: 现有错误处理框架_
  - _Requirements: 2.7_

### Phase 10: 中间件和安全增强

- [ ] 28. 增强速率限制中间件
  - File: server/api/src/middleware/rate_limit.rs (修改现有)
  - 集成新的速率限制服务
  - 添加 IP 和手机号双重限制
  - Purpose: 在中间件层执行速率限制
  - _Leverage: 现有中间件结构, rate_limiter.rs_
  - _Requirements: 3.1, 3.3_

- [ ] 29. 实现安全头中间件增强
  - File: server/api/src/middleware/security.rs (修改现有)
  - 添加 CSP、HSTS 等安全头
  - 配置 TLS 1.3 强制要求
  - Purpose: 增强API安全性
  - _Leverage: 现有安全中间件_
  - _Requirements: Security NFR_

- [ ] 30. 实现请求日志中间件
  - File: server/api/src/middleware/request_logger.rs
  - 创建结构化日志记录
  - 添加敏感数据脱敏逻辑
  - Purpose: 记录所有API请求用于调试和审计
  - _Leverage: tracing crate_
  - _Requirements: 7.7_

### Phase 11: 配置和环境管理

- [ ] 31. 添加 Twilio 配置结构
  - File: server/infra/src/config/twilio.rs
  - 创建 TwilioConfig 结构体
  - 实现从环境变量加载配置
  - Purpose: 管理Twilio服务配置
  - _Leverage: server/shared/src/config/mod.rs_
  - _Requirements: 2.9_

- [ ] 32. 添加 AWS SNS 配置结构
  - File: server/infra/src/config/aws_sns.rs
  - 创建 AwsSnsConfig 结构体
  - 实现从环境变量加载配置
  - Purpose: 管理AWS SNS服务配置
  - _Leverage: server/shared/src/config/mod.rs_
  - _Requirements: 2.10_

- [ ] 33. 更新应用配置聚合
  - File: server/api/src/config.rs (修改现有)
  - 添加认证相关配置项
  - 包含速率限制、OTP过期时间等配置
  - Purpose: 集中管理所有认证配置
  - _Leverage: 现有配置结构_
  - _Requirements: 4.1_

### Phase 12: 测试实现

- [ ] 34. 创建 OTP 加密服务单元测试
  - File: server/core/src/services/crypto/otp_encryption.rs (tests模块)
  - 测试加密/解密往返
  - 测试密钥管理
  - Purpose: 验证加密服务正确性
  - _Leverage: 标准 Rust 测试框架_
  - _Requirements: 8.1_

- [ ] 35. 创建速率限制服务单元测试
  - File: server/infra/src/cache/rate_limiter.rs (tests模块)
  - 测试限制逻辑
  - 测试计数器重置
  - Purpose: 验证速率限制功能
  - _Leverage: MockCacheService_
  - _Requirements: 3.1_

- [ ] 36. 创建验证码验证集成测试
  - File: server/core/tests/verification_integration.rs
  - 测试完整的发送-验证流程
  - 测试过期和重试逻辑
  - Purpose: 验证端到端验证流程
  - _Leverage: 测试工具函数_
  - _Requirements: 2.1, 2.2_

- [ ] 37. 创建 JWT 令牌服务单元测试
  - File: server/core/src/services/token/service.rs (tests模块)
  - 测试令牌生成和验证
  - 测试过期处理
  - Purpose: 验证JWT功能
  - _Leverage: MockTokenRepository_
  - _Requirements: 5.1, 5.7_

- [ ] 38. 创建认证 API 端点集成测试
  - File: server/api/tests/auth_integration.rs
  - 测试 send-code 端点
  - 测试 verify-code 端点
  - Purpose: 验证API端点功能
  - _Leverage: actix-web test helpers_
  - _Requirements: 1.3, 2.1_

### Phase 13: 监控和指标

- [ ] 39. 添加 Prometheus 指标收集
  - File: server/api/src/metrics/auth_metrics.rs
  - 创建认证相关指标（成功率、延迟）
  - 添加 SMS 发送指标
  - Purpose: 监控系统性能和健康状态
  - _Leverage: prometheus crate_
  - _Requirements: Monitoring NFR_

- [ ] 40. 实现健康检查端点
  - File: server/api/src/routes/health.rs
  - 添加 Redis 连接检查
  - 添加数据库连接检查
  - Purpose: 支持负载均衡器健康检查
  - _Leverage: 现有健康检查框架_
  - _Requirements: Reliability NFR_

### Phase 14: 文档和部署准备

- [ ] 41. 创建环境变量模板文件
  - File: server/.env.example
  - 列出所有必需的环境变量
  - 添加配置说明和示例值
  - Purpose: 简化部署配置
  - _Leverage: 现有 .env 格式_
  - _Requirements: 配置管理_

- [ ] 42. 创建数据库迁移运行脚本
  - File: scripts/run_migrations.sh
  - 自动运行所有待执行迁移
  - 添加回滚支持
  - Purpose: 简化数据库部署
  - _Leverage: sqlx migrate_
  - _Requirements: 数据库准备_

- [ ] 43. 更新 API 文档
  - File: docs/api/auth_endpoints.md
  - 文档化认证端点
  - 添加请求/响应示例
  - Purpose: 为前端开发提供API文档
  - _Leverage: OpenAPI 格式_
  - _Requirements: API文档_
