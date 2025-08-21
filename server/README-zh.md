# RenovEasy 后端服务器

## 📱 项目概述

RenovEasy（装修易）是一个跨平台移动应用的后端服务，为连接房主与专业装修工人的家居维护和装饰服务市场提供支持。这个基于 Rust 的后端为 iOS、Android 和 HarmonyOS 平台提供稳健、可扩展和安全的服务。

### 核心功能
- 🔐 **无密码认证**：基于手机号的认证，通过短信验证
- 🌍 **双语支持**：完整的中英文国际化支持
- 👥 **用户角色管理**：区分客户和工人的配置文件
- 🚀 **高性能**：使用 Rust 构建，实现最佳性能和安全性
- 🔄 **跨平台**：通过 FFI 为多个移动平台提供统一后端

## 🏗️ 架构设计

后端遵循**清洁架构**原则和**领域驱动设计**：

```
┌─────────────────────────────────────────────┐
│              移动应用程序                    │
│      (iOS / Android / HarmonyOS)            │
└─────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────┐
│              REST API 层                    │
│          (Actix-web 框架)                   │
├─────────────────────────────────────────────┤
│              中间件层                        │
│   (认证、CORS、速率限制、安全)               │
├─────────────────────────────────────────────┤
│            核心业务逻辑                      │
│      (服务、领域模型、规则)                  │
├─────────────────────────────────────────────┤
│            基础设施层                        │
│    (数据库、缓存、短信、外部API)             │
└─────────────────────────────────────────────┘
```

### Crate 结构
- **`api`**：REST API 端点、中间件和 HTTP 处理
- **`core`**：业务逻辑、领域模型和服务接口
- **`infra`**：基础设施实现（数据库、缓存、短信）
- **`shared`**：通用工具、类型和配置
- **`ffi`**：移动平台的外部函数接口（未来功能）

## 🚀 快速开始

您可以使用 Docker（推荐）或手动方式运行 RenovEasy 后端。

### 选项 1：Docker 部署（推荐）

#### 前置要求
- **Docker**：20.10+ 和 Docker Compose 2.0+
- **Git**：版本控制

#### 使用 Docker 快速开始

1. **克隆仓库**
```bash
git clone https://github.com/yourusername/renov-easy.git
cd renov-easy/server
```

2. **设置环境变量**
```bash
# 复制示例环境文件
cp .env.development .env

# 编辑 .env 设置您的配置
# 重要：生产环境请更改 JWT_SECRET 和数据库密码
```

3. **启动所有服务**
```bash
# 开发环境（支持热重载）
docker-compose up -d

# 或生产环境
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

4. **运行数据库迁移**
```bash
docker-compose --profile migrate up migrate
```

5. **验证服务运行状态**
```bash
# 检查服务状态
docker-compose ps

# 查看日志
docker-compose logs -f backend

# 测试健康检查端点
curl http://localhost:8080/health
```

应用程序将在 `http://localhost:8080` 可用

#### 使用 Docker 进行开发

支持热重载和调试工具的开发环境：

```bash
# 启动开发工具（phpMyAdmin、RedisInsight）
docker-compose --profile dev-tools up -d

# 访问开发服务：
# - 后端 API：http://localhost:8080
# - phpMyAdmin：http://localhost:8081
# - RedisInsight：http://localhost:8082

# 在容器中运行测试
docker-compose exec backend cargo test

# 访问后端 shell 进行调试
docker-compose exec backend /bin/bash
```

### 选项 2：手动安装

#### 前置要求
- **Rust**：1.75+ （通过 [rustup](https://rustup.rs/) 安装）
- **MySQL**：8.0+ 
- **Redis**：7.0+
- **Git**：版本控制

#### 安装步骤

1. **克隆仓库**
```bash
git clone https://github.com/yourusername/renov-easy.git
cd renov-easy/server
```

2. **安装依赖**
```bash
cargo build
```

3. **设置数据库**

使用 Docker 启动 MySQL 和 Redis：
```bash
# MySQL
docker run -d \
  --name renoveasy-mysql \
  -p 3306:3306 \
  -e MYSQL_ROOT_PASSWORD=root \
  -e MYSQL_DATABASE=renoveasy_dev \
  -e MYSQL_USER=renoveasy \
  -e MYSQL_PASSWORD=renoveasy_dev_2025 \
  mysql:8

# Redis
docker run -d \
  --name renoveasy-redis \
  -p 6379:6379 \
  redis:7-alpine
```

4. **运行数据库迁移**
```bash
# 如果还没有安装 sqlx-cli，请先安装
cargo install sqlx-cli --no-default-features --features mysql

# 运行迁移
sqlx migrate run --database-url "mysql://renoveasy:renoveasy_dev_2025@localhost:3306/renoveasy_dev"
```

5. **配置环境**
```bash
# 复制开发环境配置文件
cp .env.development .env

# 编辑 .env 以匹配您的本地设置
# 需要验证的关键配置：
# - DATABASE_URL
# - REDIS_URL
# - JWT_SECRET（生产环境需要更改）
```

## 🏃 运行服务器

### 开发模式
```bash
# 使用热重载运行
cargo watch -x "run --bin api"

# 或直接运行
cargo run --bin api
```

服务器将在 `http://localhost:8080` 启动

### 生产构建
```bash
# 构建优化的二进制文件
cargo build --release

# 运行生产二进制文件
./target/release/api
```

## 🔧 配置说明

### 环境变量

应用程序使用环境变量进行配置。主要变量包括：

| 变量 | 描述 | 默认值 |
|------|------|--------|
| `ENVIRONMENT` | 环境模式 (development/staging/production) | `development` |
| `SERVER_HOST` | 服务器主机地址 | `127.0.0.1` |
| `SERVER_PORT` | 服务器端口 | `8080` |
| `DATABASE_URL` | MySQL 连接字符串 | 必需 |
| `REDIS_URL` | Redis 连接字符串 | 必需 |
| `JWT_SECRET` | JWT 签名密钥 | 必需 |
| `SMS_PROVIDER` | 短信服务提供商 (mock/twilio/aws_sns) | `mock` |

完整的配置选项列表请参见 `.env.development`。

### 配置优先级
1. 环境变量
2. 项目根目录的 `.env` 文件
3. 默认值（仅开发环境）

## 📚 API 文档

### 认证端点

| 方法 | 端点 | 描述 |
|------|------|------|
| POST | `/api/v1/auth/send-code` | 发送短信验证码 |
| POST | `/api/v1/auth/verify-code` | 验证短信验证码并登录 |
| POST | `/api/v1/auth/select-type` | 选择用户类型（客户/工人） |
| POST | `/api/v1/auth/refresh` | 刷新访问令牌 |
| POST | `/api/v1/auth/logout` | 登出并撤销令牌 |

### 请求/响应示例

**发送验证码**
```bash
curl -X POST http://localhost:8080/api/v1/auth/send-code \
  -H "Content-Type: application/json" \
  -d '{
    "phone": "0412345678",
    "country_code": "+61"
  }'
```

**验证验证码**
```bash
curl -X POST http://localhost:8080/api/v1/auth/verify-code \
  -H "Content-Type: application/json" \
  -d '{
    "phone": "0412345678",
    "country_code": "+61",
    "code": "123456"
  }'
```

详细的 API 文档请参见 [API.md](./API.md) 或在启用 Swagger 时访问 `/api/docs`。

## 🧪 测试

### 运行所有测试
```bash
cargo test
```

### 运行特定类别的测试
```bash
# 仅单元测试
cargo test --lib

# 仅集成测试
cargo test --test '*'

# 带输出用于调试
cargo test -- --nocapture

# 运行特定 crate 的测试
cargo test -p core
cargo test -p api
cargo test -p infra
```

### 测试覆盖率
```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 生成覆盖率报告
cargo tarpaulin --out Html
```

## 🔨 开发工作流

### 代码格式化
```bash
# 格式化所有代码
cargo fmt

# 检查格式化但不做更改
cargo fmt -- --check
```

### 代码检查
```bash
# 运行 clippy 进行代码质量检查
cargo clippy -- -D warnings
```

### 安全审计
```bash
# 检查安全漏洞
cargo audit
```

### 数据库开发

**创建新迁移**
```bash
sqlx migrate add <migration_name>
```

**回滚上一次迁移**
```bash
sqlx migrate revert
```

### 调试

1. **启用调试日志**
   - 在 `.env` 中设置 `LOG_LEVEL=debug`
   - 设置 `LOG_SQL_QUERIES=true` 以查看数据库查询

2. **开发环境使用模拟短信**
   - 设置 `SMS_PROVIDER=mock` 以避免发送真实短信
   - 验证码将打印到控制台

3. **使用 curl 或 Postman 测试**
   - 从 `docs/postman/` 导入 Postman 集合
   - 或使用提供的 curl 示例

## 📦 项目结构

```
server/
├── api/                  # REST API 服务器
│   ├── src/
│   │   ├── routes/       # API 端点
│   │   ├── middleware/   # 认证、CORS、速率限制
│   │   ├── dto/          # 请求/响应模型
│   │   └── config.rs     # 配置管理
│   └── tests/            # API 集成测试
├── core/                 # 业务逻辑
│   └── src/
│       ├── domain/       # 实体和值对象
│       ├── services/     # 业务服务
│       └── repositories/ # 仓储接口
├── infra/                # 基础设施
│   └── src/
│       ├── database/     # MySQL 实现
│       ├── cache/        # Redis 实现
│       └── sms/          # 短信服务
├── shared/               # 共享工具
│   └── src/
│       ├── config/       # 配置类型
|       ├── errors/       # 通用错误
│       ├── types/        # 通用类型
│       └── utils/        # 工具函数
└── migrations/           # 数据库迁移
```

## 🤝 贡献指南

1. 创建功能分支：`git checkout -b feature/your-feature`
2. 进行更改并添加测试
3. 确保所有测试通过：`cargo test`
4. 格式化代码：`cargo fmt`
5. 检查代码质量：`cargo clippy`
6. 使用约定式提交：`feat(scope): description`
7. 推送并创建拉取请求

## 🚀 部署

### Docker 部署

项目包含全面的 Docker 支持，为开发和生产环境提供不同的配置。

#### Docker 文件概览

- **`Dockerfile`**：多阶段构建优化的 Rust 应用
  - `builder` 阶段：设置构建环境
  - `dependencies` 阶段：缓存 Cargo 依赖
  - `build` 阶段：编译应用程序
  - `runtime` 阶段：最小化生产镜像
  - `development` 阶段：包含开发工具

- **`docker-compose.yml`**：包含 MySQL、Redis 和后端服务的基础配置
- **`docker-compose.override.yml`**：开发环境覆盖配置（开发时自动加载）
- **`docker-compose.prod.yml`**：生产环境特定配置，包含安全加固

#### 开发环境部署

```bash
# 以开发模式启动所有服务
docker-compose up -d

# 包含开发工具（phpMyAdmin、RedisInsight）
docker-compose --profile dev-tools up -d

# 代码更改后重新构建
docker-compose up -d --build backend

# 查看实时日志
docker-compose logs -f backend

# 停止所有服务
docker-compose down

# 重置所有内容（包括数据卷）
docker-compose down -v
```

#### 生产环境部署

```bash
# 构建生产镜像
docker build --target production -t renoveasy-backend:latest .

# 使用生产配置部署
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d

# 部署特定版本
IMAGE_TAG=v1.0.0 docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d

# 启用自动备份
docker-compose -f docker-compose.yml -f docker-compose.prod.yml --profile backup up -d

# 扩展后端服务
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d --scale backend=3
```

#### 容器管理

```bash
# 查看运行中的容器
docker-compose ps

# 在容器中执行命令
docker-compose exec backend cargo test
docker-compose exec mysql mysqldump -u root -p renoveasy > backup.sql

# 访问容器 shell
docker-compose exec backend /bin/bash
docker-compose exec mysql mysql -u root -p

# 监控资源使用
docker stats

# 清理未使用的资源
docker system prune -a
```

### 环境特定配置

- **开发环境**：使用 `.env.development` 和模拟服务
  - 模拟短信提供商
  - 简单密码
  - 启用调试日志
  - 支持热重载

- **测试环境**：使用 `.env.staging` 和测试数据库
  - 真实短信提供商（测试账户）
  - 中等安全设置
  - Info 级别日志

- **生产环境**：使用 `.env.production` 和生产服务
  - 生产短信提供商
  - 强密码和密钥
  - 启用安全头
  - 优化的日志记录

⚠️ **重要提示**：永远不要提交 `.env.production` 或任何包含真实密钥的文件！

## 🔒 安全考虑

- 生产环境中所有端点使用 HTTPS
- JWT 令牌过期时间：访问令牌 15 分钟，刷新令牌 7 天
- 手机号在存储前进行哈希处理
- 速率限制防止滥用（每个手机号每小时 3 次短信）
- 通过参数化查询防止 SQL 注入
- 所有端点进行输入验证

## 📊 监控

- **健康检查**：`GET /health`
- **指标**：启用时在端口 9090 可用
- **日志记录**：生产环境使用结构化 JSON 日志
- **错误跟踪**：集成 Sentry（可选）

## 🐛 故障排除

### 常见问题

1. **数据库连接失败**
   - 验证 MySQL 正在运行：`docker ps`
   - 检查 `.env` 中的 DATABASE_URL
   - 确保数据库存在且用户有权限

2. **Redis 连接失败**
   - 验证 Redis 正在运行：`docker ps`
   - 检查 `.env` 中的 REDIS_URL
   - 测试连接：`redis-cli ping`

3. **短信未发送**
   - 检查 SMS_PROVIDER 配置
   - 开发环境使用 `mock` 提供商
   - 验证生产提供商的 API 密钥

4. **端口已被使用**
   - 在 `.env` 中更改 SERVER_PORT
   - 或停止冲突的服务

## 📝 许可证

本项目为专有软件。保留所有权利。

## 🆘 支持

如有问题和疑问：
- 在 GitHub 仓库中创建 issue
- 联系开发团队
- 查看 [API 文档](./API.md)

---
