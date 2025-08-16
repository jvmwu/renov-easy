# 代码重构架构设计说明

## 概述
本文档记录了 Renov Easy 项目中 `server/core` 层的代码重构方案，特别是 `services` 和 `repositories` 层的模块化拆分设计。

## 重构背景
原始代码将所有相关功能都放在单个文件中，导致：
- 文件过大，难以维护（单文件可能超过 500-800 行）
- 职责不清晰，违反单一职责原则
- 测试代码与业务代码混杂
- 配置、工具函数与核心逻辑耦合

## 重构方案

### 1. Repositories 层重构

#### 原始结构
```
repositories/
├── token_repository.rs     # 包含 trait 定义 + mock 实现 + 测试
└── user_repository.rs      # 包含 trait 定义 + mock 实现 + 测试
```

#### 重构后结构
```
repositories/
├── token/
│   ├── mod.rs          # 模块导出
│   ├── trait.rs        # TokenRepository trait 定义
│   ├── repository.rs   # 实现占位符（原 mysql.rs，已重命名）
│   ├── mock.rs         # Mock 实现（用于测试）
│   └── tests/          # 单元测试
│       ├── mod.rs
│       └── mock_tests.rs
└── user/
    ├── mod.rs          # 模块导出
    ├── trait.rs        # UserRepository trait 定义
    ├── repository.rs   # 实现占位符（原 mysql.rs，已重命名）
    ├── mock.rs         # Mock 实现（用于测试）
    └── tests/          # 单元测试
        ├── mod.rs
        └── mock_tests.rs
```

#### 文件职责说明

##### trait.rs - 接口定义
```rust
// 定义抽象接口，不包含任何实现
pub trait TokenRepository: Send + Sync {
    async fn save_refresh_token(&self, token: RefreshToken) -> Result<RefreshToken, DomainError>;
    async fn find_refresh_token(&self, token_hash: &str) -> Result<Option<RefreshToken>, DomainError>;
    // ... 其他方法
}
```
**作用**：
- 定义业务层的抽象接口
- 与具体实现技术无关
- 所有实现必须遵循这个契约

##### repository.rs - 实现占位符
```rust
// 占位符，实际实现在 infrastructure 层
pub struct MySqlTokenRepository;

// 实际实现位置：
// server/infrastructure/src/database/mysql/token_repository_impl.rs
```
**作用**：
- 在 core 层提供类型定义，便于编译通过
- 实际的数据库实现在 infrastructure 层
- 保持架构分层：core 层不依赖具体数据库技术

##### mock.rs - 测试用 Mock 实现
```rust
pub struct MockTokenRepository {
    tokens: Arc<RwLock<HashMap<String, RefreshToken>>>,
}

impl TokenRepository for MockTokenRepository {
    // 内存中的实现，用于测试
}
```
**作用**：
- 提供轻量级的内存实现
- 用于单元测试，不需要真实数据库
- 测试运行更快，更可控

### 2. Services 层重构

#### 原始结构
```
services/
├── auth_service.rs         # 所有认证相关代码在一个文件
├── token_service.rs        # 所有令牌相关代码在一个文件
└── verification_service.rs # 所有验证相关代码在一个文件
```

#### 重构后结构
```
services/
├── auth/
│   ├── mod.rs          # 模块导出
│   ├── service.rs      # AuthService 实现（包含业务逻辑）
│   ├── config.rs       # 配置结构
│   ├── phone_utils.rs  # 手机号处理工具
│   ├── rate_limiter.rs # 速率限制
│   └── tests/          # 测试目录
│       ├── mocks.rs    # Mock 实现
│       └── service_tests.rs
│
├── token/
│   ├── mod.rs          # 模块导出
│   ├── service.rs      # TokenService 实现
│   ├── config.rs       # 配置结构
│   └── tests/
│       └── service_tests.rs
│
└── verification/
    ├── mod.rs          # 模块导出
    ├── service.rs      # VerificationService 实现
    ├── config.rs       # 配置结构
    ├── traits.rs       # 外部依赖的 trait（SmsService, CacheService）
    ├── types.rs        # 类型定义
    └── tests/
        ├── mocks.rs
        └── service_tests.rs
```

### 3. Services 与 Repositories 的关键差异

| 方面 | Repositories 层 | Services 层 |
|------|----------------|-------------|
| **核心职责** | 数据访问抽象 | 业务逻辑实现 |
| **trait 定义** | 必须有（定义数据访问接口） | 通常不需要（业务逻辑是确定的） |
| **实现位置** | infrastructure 层 | core 层 |
| **为什么** | 数据存储是技术细节 | 业务规则是领域核心 |
| **示例** | `trait UserRepository` | `struct AuthService` |

#### 为什么 Services 不需要 trait？
- Services 包含的是**业务逻辑**，这些规则是确定的
- 业务规则（如"每小时最多发送3次验证码"）是领域的一部分
- 不需要多种实现，因为业务规则是唯一的

#### 为什么 Repositories 需要 trait？
- Repositories 是**数据访问的抽象**
- 可能有多种存储实现（MySQL、PostgreSQL、MongoDB、Redis）
- 核心层定义接口，基础设施层提供具体实现

#### 特殊情况：verification/traits.rs
```rust
// 这些 trait 不是 VerificationService 的抽象
// 而是它依赖的外部服务的抽象
pub trait SmsServiceTrait { }    // 短信服务（阿里云、腾讯云等）
pub trait CacheServiceTrait { }  // 缓存服务（Redis、Memcached等）
```

## 架构层次图

```
┌─────────────────────────────────────────┐
│            Core 层（核心业务）            │
│                                         │
│  ┌─────────────────────────────────┐    │
│  │     Services（业务逻辑）         │    │
│  │  - AuthService（具体实现）       │    │
│  │  - TokenService（具体实现）      │    │
│  │  - 包含业务规则和流程            │    │
│  └─────────────────────────────────┘    │
│                ↓ 依赖                    │
│  ┌─────────────────────────────────┐    │
│  │   Repositories（数据访问抽象）    │    │
│  │  - UserRepository（trait）       │    │
│  │  - TokenRepository（trait）      │    │
│  │  - 只定义接口，不包含实现         │    │
│  └─────────────────────────────────┘    │
└─────────────────────────────────────────┘
                    ↓ 实现
┌─────────────────────────────────────────┐
│       Infrastructure 层（技术实现）       │
│                                         │
│  ┌─────────────────────────────────┐    │
│  │    数据库实现                    │    │
│  │  - MySqlUserRepository          │    │
│  │  - MySqlTokenRepository         │    │
│  │  - 包含 SQL 语句和数据库操作      │    │
│  └─────────────────────────────────┘    │
└─────────────────────────────────────────┘
```

## 重构收益

### 1. 单一职责原则
- 每个文件只负责一个明确的功能
- trait.rs：定义接口
- service.rs：实现业务逻辑
- config.rs：管理配置
- mock.rs：提供测试实现

### 2. 更好的代码组织
```
原来：一个大文件包含所有内容（500-800行）
现在：多个小文件，每个专注一个方面（50-200行）
```

### 3. 测试隔离
- 测试代码独立在 tests/ 目录
- Mock 实现与生产代码分离
- 条件编译更清晰（#[cfg(test)]）

### 4. 依赖注入更灵活
```rust
// 可以轻松切换不同的实现
fn create_service(repo: impl TokenRepository) {
    // 测试时注入 MockTokenRepository
    // 生产时注入 MySqlTokenRepository
}
```

### 5. 维护性提升
- 需要修改配置？只看 config.rs
- 需要调整工具函数？只改 phone_utils.rs
- 需要增加新功能？创建新文件，不影响现有代码

## 命名规范改进

### 原问题
在 core 层使用 `mysql.rs` 这样的技术实现名称不合适，因为：
- Core 层应该与技术无关
- 暴露了实现细节
- 违反了依赖倒置原则

### 解决方案
将 `mysql.rs` 重命名为 `repository.rs`：
- 更符合业务语义
- 不暴露技术细节
- 明确表示这是仓储模式的占位符

## 实施步骤

1. **创建新的目录结构**
   - 为每个模块创建独立目录
   - 创建 tests 子目录

2. **拆分代码**
   - 将 trait 定义移到 trait.rs
   - 将配置移到 config.rs
   - 将工具函数独立出来
   - 将测试移到 tests/

3. **更新导入路径**
   - 修改所有引用这些模块的文件
   - 更新 mod.rs 的导出

4. **运行测试验证**
   - 确保所有测试通过
   - 验证编译无误

## 注意事项

1. **保持向后兼容**
   - 通过 mod.rs 重新导出公共 API
   - 外部模块的使用方式不变

2. **条件编译**
   ```rust
   #[cfg(test)]
   pub mod mock;  // 只在测试时编译
   ```

3. **文档更新**
   - 每个模块都要有清晰的文档说明
   - 更新 README 反映新的结构

## 总结

这次重构遵循了以下原则：
- **洋葱架构**：业务逻辑在内层，技术实现在外层
- **依赖倒置**：高层模块不依赖低层模块，都依赖抽象
- **单一职责**：每个模块/文件只负责一件事
- **开闭原则**：对扩展开放，对修改关闭

通过这种重构，代码的可维护性、可测试性和可扩展性都得到了显著提升。