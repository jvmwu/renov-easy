# Requirements Document - 无密码认证系统 (Passwordless Authentication System)

## Introduction

无密码认证系统是 RenovEasy 平台的核心安全基础设施，通过基于 SMS 的 OTP (一次性密码) 验证实现安全便捷的用户身份认证。该系统消除了传统密码的复杂性和安全隐患，采用手机号验证确保用户身份真实性，并通过 JWT 令牌机制管理会话状态。系统集成 Twilio 和 AWS SNS 作为 SMS 服务提供商，确保消息投递的可靠性。系统配备完善的速率限制、审计日志和安全防护机制。

## Alignment with Product Vision

该功能直接支持 RenovEasy "让小型家庭维修和装饰变得简单易用" 的核心价值主张：

- **降低使用门槛**: 用户无需记忆密码即可快速访问平台，特别适合非技术用户群体
- **建立信任基础**: 手机号验证确保真实用户身份，这对连接房主和工人的市场平台至关重要
- **支持全球化**: 基于 SMS 的认证支持中国（+86）和澳大利亚（+61）等目标市场
- **加速用户增长**: 新用户可在几分钟内完成注册并开始使用平台
- **保障交易安全**: 通过验证身份和安全令牌保护客户和工人的利益

## Requirements

### Requirement 1: 手机号验证流程 (Phone Verification Flow)

**User Story:** 作为新用户，我希望使用手机号注册并通过短信接收验证码，这样我可以快速创建账户而无需管理密码。

#### Acceptance Criteria

1. WHEN 用户输入手机号 THEN 系统 SHALL 根据国际标准（E.164格式）验证格式正确性
2. IF 手机号格式无效 THEN 系统 SHALL 用用户选择的语言显示清晰的错误信息
3. WHEN 提交有效手机号 AND 未超过速率限制 THEN 系统 SHALL 在10秒内发送包含6位验证码的短信
4. IF 手机号已在系统中存在 THEN 系统 SHALL 切换到登录流程而非注册
5. WHEN SMS发送失败 THEN 系统 SHALL 提供适当反馈并建议30秒后重试
6. IF 用户来自中国 THEN 系统 SHALL 支持中国手机格式（+86前缀，11位号码）
7. WHEN 用户来自澳大利亚 THEN 系统 SHALL 支持澳大利亚手机格式（+61前缀）
8. IF 手机号未注册 THEN 系统 SHALL 创建新用户记录并标记为待激活状态

### Requirement 2: SMS OTP 发送与验证 (SMS OTP Generation and Validation)

**User Story:** 作为登录用户，我希望接收一个在合理时间后过期的安全验证码，这样即使有人截获短信我的账户也能保持安全。

#### Acceptance Criteria

1. WHEN 生成验证码 THEN 系统 SHALL 创建密码学安全的6位数字代码
2. IF 验证码在5分钟内未使用 THEN 系统 SHALL 自动使其失效
3. WHEN 用户输入错误代码 THEN 系统 SHALL 增加尝试计数器并提供反馈
4. IF 用户连续输入错误代码3次 THEN 系统 SHALL 锁定该手机号验证1小时
5. WHEN 验证码成功使用 THEN 系统 SHALL 立即使其失效防止重复使用
6. IF 请求多个验证码 THEN 系统 SHALL 使所有之前未使用的验证码失效
7. WHEN 验证代码时 THEN 系统 SHALL 使用恒定时间比较防止时序攻击
8. IF 验证码过期 THEN 系统 SHALL 返回明确的过期错误信息
9. WHEN 发送SMS THEN 系统 SHALL 使用 Twilio 作为主要SMS服务提供商
10. IF Twilio 服务失败 THEN 系统 SHALL 自动切换到 AWS SNS 作为备用提供商
11. WHEN 验证码存储到Redis THEN 系统 SHALL 使用加密存储保护验证码安全

### Requirement 3: 速率限制 (Rate Limiting)

**User Story:** 作为系统管理员，我希望认证系统能防止滥用和垃圾请求，确保平台稳定性和控制SMS成本。

#### Acceptance Criteria

1. WHEN 手机号请求验证码 THEN 系统 SHALL 限制为每小时3次
2. IF 手机号超过速率限制 THEN 系统 SHALL 返回错误并显示剩余冷却时间
3. WHEN IP地址发送多个请求 THEN 系统 SHALL 限制每小时所有手机号总共10次验证尝试
4. IF 检测到可疑模式 THEN 系统 SHALL 触发额外安全措施（未来实现CAPTCHA）
5. WHEN 达到速率限制 THEN 系统 SHALL 记录事件用于安全监控
6. IF 检测到IP范围的系统性滥用 THEN 系统 SHALL 支持临时IP封禁
7. WHEN 速率限制触发 THEN 系统 SHALL 在审计日志中记录详细信息
8. IF 速率限制数据存储 THEN 系统 SHALL 使用Redis进行快速访问

### Requirement 4: 验证码过期管理 (OTP Expiry Management)

**User Story:** 作为安全管理员，我希望验证码有严格的生命周期管理，确保系统安全性和用户体验平衡。

#### Acceptance Criteria

1. WHEN 验证码生成 THEN 系统 SHALL 设置5分钟有效期
2. IF 验证码即将过期（剩余1分钟）THEN 系统 SHALL 允许用户请求新验证码
3. WHEN 验证码过期 THEN 系统 SHALL 自动从Redis缓存中删除
4. IF 用户尝试使用过期验证码 THEN 系统 SHALL 返回明确的过期提示
5. WHEN 新验证码生成 THEN 系统 SHALL 立即使旧验证码失效
6. IF 系统检测到验证码被暴力破解 THEN 系统 SHALL 立即使该验证码失效
7. WHEN 验证码在Redis中存储 THEN 系统 SHALL 使用TTL确保自动过期

### Requirement 5: JWT 令牌生成 (JWT Token Generation)

**User Story:** 作为已验证的用户，我希望获得安全的访问令牌，以便在后续请求中证明我的身份。

#### Acceptance Criteria

1. WHEN 用户成功验证手机 THEN 系统 SHALL 颁发有效期15分钟的JWT访问令牌
2. IF 用户成功验证 THEN 系统 SHALL 同时颁发有效期30天的刷新令牌
3. WHEN 生成JWT令牌 THEN 系统 SHALL 使用RS256算法进行签名
4. IF 生成令牌 THEN 系统 SHALL 在令牌中包含用户ID、手机号哈希、过期时间等必要声明
5. WHEN 存储刷新令牌 THEN 系统 SHALL 在数据库refresh_tokens表中记录令牌哈希值
6. IF 令牌生成成功 THEN 系统 SHALL 返回访问令牌、刷新令牌和过期时间
7. WHEN 令牌过期 THEN 系统 SHALL 拒绝访问并返回401未授权错误

### Requirement 6: 防暴力破解 (Brute Force Protection)

**User Story:** 作为平台运营者，我希望系统能够识别并阻止暴力破解尝试，保护用户账户安全。

#### Acceptance Criteria

1. WHEN 检测到同一手机号的多次失败尝试 THEN 系统 SHALL 增加延迟响应时间
2. IF 失败尝试超过3次 THEN 系统 SHALL 锁定该手机号1小时
3. WHEN 检测到同一IP的异常活动 THEN 系统 SHALL 触发IP级别的限制
4. IF 发现分布式攻击模式 THEN 系统 SHALL 启动全局防护机制
5. WHEN 暴力破解被检测 THEN 系统 SHALL 生成安全告警
6. IF 账户被锁定 THEN 系统 SHALL 通过备用渠道通知用户（未来功能）

### Requirement 7: 审计日志记录 (Audit Logging)

**User Story:** 作为安全官员，我希望所有认证事件都被详细记录，以便调查安全事件并确保合规性。

#### Acceptance Criteria

1. WHEN 任何认证事件发生 THEN 系统 SHALL 创建不可变的审计日志条目
2. IF 登录尝试失败 THEN 系统 SHALL 记录失败原因、IP地址、设备信息和时间戳
3. WHEN 成功登录 THEN 系统 SHALL 记录用户ID、IP、设备信息和时间戳
4. IF 触发速率限制 THEN 系统 SHALL 记录手机号（脱敏）、IP和违规类型
5. WHEN 令牌被生成 THEN 系统 SHALL 记录令牌ID、用户ID和生成时间
6. IF 审计日志达到90天 THEN 系统 SHALL 根据保留策略进行归档
7. WHEN 记录敏感信息 THEN 系统 SHALL 对手机号进行脱敏（仅显示后4位）
8. IF 审计日志写入 THEN 系统 SHALL 使用auth_audit_log表进行持久化

### Requirement 8: 验证码加密存储 (OTP Encryption Storage)

**User Story:** 作为安全架构师，我希望验证码在存储时被加密，即使缓存被泄露也不会暴露明文验证码。

#### Acceptance Criteria

1. WHEN 验证码生成 THEN 系统 SHALL 使用AES-256-GCM算法加密
2. IF 验证码存储到Redis THEN 系统 SHALL 只存储加密后的密文
3. WHEN 验证码需要比对 THEN 系统 SHALL 先解密再进行比较
4. IF 加密密钥轮换 THEN 系统 SHALL 支持平滑过渡不影响现有验证码
5. WHEN 存储验证码元数据 THEN 系统 SHALL 包含创建时间、尝试次数、过期时间
6. IF Redis连接失败 THEN 系统 SHALL 降级到数据库存储（带告警）

## Non-Functional Requirements

### Performance

- SMS 发送延迟 SHALL 小于10秒（95%的请求）
- 验证码验证响应时间 SHALL 小于50ms
- 令牌生成和验证 SHALL 在50ms内完成
- 速率限制检查 SHALL 在10ms内完成
- 用户查询的数据库操作 SHALL 在100ms内完成
- 系统 SHALL 支持1000个并发认证请求
- Redis 缓存操作 SHALL 在5ms内完成

### Security

- 所有手机号 SHALL 使用SHA-256哈希后存储在数据库
- 验证码 SHALL 使用密码学安全的随机数生成器（CSPRNG）
- JWT令牌 SHALL 使用RS256（RSA签名配SHA-256）算法
- 所有认证端点 SHALL 使用HTTPS/TLS 1.3
- 失败的认证尝试 SHALL 使用恒定时间响应防止时序攻击
- 敏感数据 SHALL 在日志中脱敏（仅显示手机号后4位）
- 系统 SHALL 实施OWASP认证最佳实践
- 验证码 SHALL 在Redis中使用AES-256-GCM加密存储

### Reliability

- 认证服务 SHALL 保持99.9%的正常运行时间
- SMS提供商故障切换 SHALL 在主服务（Twilio）失败后30秒内激活备用服务（AWS SNS）
- 系统 SHALL 同时集成 Twilio 和 AWS SNS，实现自动故障转移
- 数据库连接池 SHALL 自动从临时故障中恢复
- 缓存层故障 SHALL 不影响认证（优雅降级到数据库）
- 系统 SHALL 优雅处理SMS提供商（Twilio/AWS SNS）的速率限制
- 备用认证方法 SHALL 在SMS中断期间可用（未来功能）

### Usability

- 错误消息 SHALL 以用户选择的语言清晰且可操作地显示
- 手机号输入 SHALL 支持带国家代码选择的国际格式
- 验证码输入 SHALL 在移动设备上支持从SMS自动填充
- 系统 SHALL 清晰显示验证码剩余有效时间
- 加载状态 SHALL 在SMS发送和验证期间显示
- 成功/失败反馈 SHALL 立即且清晰
- 系统 SHALL 支持无障碍标准（WCAG 2.1 Level AA）

### Scalability

- 系统 SHALL 支持水平扩展以应对增长
- Redis缓存 SHALL 支持集群以实现高可用性
- 数据库 SHALL 支持读副本进行查询分发
- SMS发送 SHALL 使用消息队列进行异步处理
- 系统 SHALL 支持多区域部署服务全球用户

### Internationalization

- 系统 SHALL 支持中文和英文的所有消息
- 手机号验证 SHALL 支持所有国际格式
- 错误消息 SHALL 根据用户语言偏好进行本地化
- SMS模板 SHALL 提供多语言版本
- 日期/时间显示 SHALL 遵循用户的区域设置

### Monitoring and Observability

- 所有认证事件 SHALL 生成结构化日志
- 关键指标 SHALL 暴露用于监控（成功率、延迟、错误）
- 系统 SHALL 支持分布式追踪用于请求调试
- 异常认证模式 SHALL 配置告警
- 仪表板 SHALL 显示实时认证指标
- 系统 SHALL 支持与Prometheus/Grafana集成
- SMS服务（Twilio/AWS SNS）的成功率和延迟 SHALL 被持续监控

### Technology Stack Integration

- **SMS服务**: Twilio（主）+ AWS SNS（备份）双服务商集成
- **缓存层**: Redis 用于存储验证码和速率限制数据
- **数据库**: MySQL 用于用户数据持久化（users, verification_codes, refresh_tokens, auth_audit_log表）
- **认证**: JWT (RS256) 用于令牌管理
- **加密**: AES-256-GCM 用于验证码加密存储
- **监控**: Prometheus + Grafana 用于指标收集和可视化
- **日志**: 结构化日志with审计追踪