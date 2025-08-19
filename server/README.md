# RenovEasy Backend Server

## 📱 Project Overview

RenovEasy (装修易) is a cross-platform mobile application backend that powers a marketplace connecting homeowners with professional renovation workers for home maintenance and decoration services. This Rust-based backend provides robust, scalable, and secure services for iOS, Android, and HarmonyOS platforms.

### Key Features
- 🔐 **Passwordless Authentication**: Phone number-based authentication with SMS verification
- 🌍 **Bilingual Support**: Full internationalization for Chinese and English
- 👥 **User Role Management**: Distinct customer and worker profiles
- 🚀 **High Performance**: Built with Rust for optimal performance and safety
- 🔄 **Cross-Platform**: Unified backend for multiple mobile platforms via FFI

## 🏗️ Architecture

The backend follows **Clean Architecture** principles with **Domain-Driven Design**:

```
┌─────────────────────────────────────────────┐
│            Mobile Applications              │
│    (iOS / Android / HarmonyOS)             │
└─────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────┐
│              REST API Layer                 │
│         (Actix-web Framework)               │
├─────────────────────────────────────────────┤
│           Middleware Layer                  │
│  (Auth, CORS, Rate Limiting, Security)      │
├─────────────────────────────────────────────┤
│         Core Business Logic                 │
│    (Services, Domain Models, Rules)         │
├─────────────────────────────────────────────┤
│        Infrastructure Layer                 │
│   (Database, Cache, SMS, External APIs)     │
└─────────────────────────────────────────────┘
```

### Crate Structure
- **`api`**: REST API endpoints, middleware, and HTTP handling
- **`core`**: Business logic, domain models, and service interfaces
- **`infra`**: Infrastructure implementations (database, cache, SMS)
- **`shared`**: Common utilities, types, and configuration
- **`ffi`**: Foreign Function Interface for mobile platforms (future)

## 🚀 Quick Start

You can run the RenovEasy backend either with Docker (recommended) or manually.

### Option 1: Docker Deployment (Recommended)

#### Prerequisites
- **Docker**: 20.10+ and Docker Compose 2.0+
- **Git**: For version control

#### Quick Start with Docker

1. **Clone the repository**
```bash
git clone https://github.com/yourusername/renov-easy.git
cd renov-easy/server
```

2. **Set up environment variables**
```bash
# Copy the example environment file
cp .env.development .env

# Edit .env to set your configuration
# Important: Change JWT_SECRET and database passwords for production
```

3. **Start all services**
```bash
# Development environment (with hot reload)
docker-compose up -d

# Or production environment
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

4. **Run database migrations**
```bash
docker-compose --profile migrate up migrate
```

5. **Verify the services are running**
```bash
# Check service status
docker-compose ps

# View logs
docker-compose logs -f backend

# Test the health endpoint
curl http://localhost:8080/health
```

The application will be available at `http://localhost:8080`

#### Development with Docker

For development with hot reload and debugging tools:

```bash
# Start with development tools (phpMyAdmin, RedisInsight)
docker-compose --profile dev-tools up -d

# Access development services:
# - Backend API: http://localhost:8080
# - phpMyAdmin: http://localhost:8081
# - RedisInsight: http://localhost:8082

# Run tests in container
docker-compose exec backend cargo test

# Access backend shell for debugging
docker-compose exec backend /bin/bash
```

### Option 2: Manual Installation

#### Prerequisites
- **Rust**: 1.75+ (install via [rustup](https://rustup.rs/))
- **MySQL**: 8.0+ 
- **Redis**: 7.0+
- **Git**: For version control

#### Installation

1. **Clone the repository**
```bash
git clone https://github.com/yourusername/renov-easy.git
cd renov-easy/server
```

2. **Install dependencies**
```bash
cargo build
```

3. **Set up databases**

Start MySQL and Redis using Docker:
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

4. **Run database migrations**
```bash
# Install sqlx-cli if not already installed
cargo install sqlx-cli --no-default-features --features mysql

# Run migrations
sqlx migrate run --database-url "mysql://renoveasy:renoveasy_dev_2025@localhost:3306/renoveasy_dev"
```

5. **Configure environment**
```bash
# Copy the development environment file
cp .env.development .env

# Edit .env to match your local setup
# Key configurations to verify:
# - DATABASE_URL
# - REDIS_URL
# - JWT_SECRET (change for production)
```

## 🏃 Running the Server

### Development Mode
```bash
# Run with hot reload
cargo watch -x "run --bin api"

# Or run directly
cargo run --bin api
```

The server will start on `http://localhost:8080`

### Production Build
```bash
# Build optimized binary
cargo build --release

# Run the production binary
./target/release/api
```

## 🔧 Configuration

### Environment Variables

The application uses environment variables for configuration. Key variables include:

| Variable | Description | Default |
|----------|-------------|---------|
| `ENVIRONMENT` | Environment mode (development/staging/production) | `development` |
| `SERVER_HOST` | Server host address | `127.0.0.1` |
| `SERVER_PORT` | Server port | `8080` |
| `DATABASE_URL` | MySQL connection string | Required |
| `REDIS_URL` | Redis connection string | Required |
| `JWT_SECRET` | Secret key for JWT signing | Required |
| `SMS_PROVIDER` | SMS service provider (mock/twilio/aws_sns) | `mock` |

See `.env.development` for a complete list of configuration options.

### Configuration Precedence
1. Environment variables
2. `.env` file in project root
3. Default values (development only)

## 📚 API Documentation

### Authentication Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/auth/send-code` | Send SMS verification code |
| POST | `/api/v1/auth/verify-code` | Verify SMS code and login |
| POST | `/api/v1/auth/select-type` | Select user type (customer/worker) |
| POST | `/api/v1/auth/refresh` | Refresh access token |
| POST | `/api/v1/auth/logout` | Logout and revoke tokens |

### Request/Response Examples

**Send Verification Code**
```bash
curl -X POST http://localhost:8080/api/v1/auth/send-code \
  -H "Content-Type: application/json" \
  -d '{
    "phone": "0412345678",
    "country_code": "+61"
  }'
```

**Verify Code**
```bash
curl -X POST http://localhost:8080/api/v1/auth/verify-code \
  -H "Content-Type: application/json" \
  -d '{
    "phone": "0412345678",
    "country_code": "+61",
    "code": "123456"
  }'
```

For detailed API documentation, see [API.md](./API.md) or visit `/api/docs` when Swagger is enabled.

## 🧪 Testing

### Run All Tests
```bash
cargo test
```

### Run Specific Test Categories
```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test '*'

# With output for debugging
cargo test -- --nocapture

# Run tests for a specific crate
cargo test -p core
cargo test -p api
cargo test -p infra
```

### Test Coverage
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html
```

## 🔨 Development Workflow

### Code Formatting
```bash
# Format all code
cargo fmt

# Check formatting without changes
cargo fmt -- --check
```

### Linting
```bash
# Run clippy for code quality checks
cargo clippy -- -D warnings
```

### Security Audit
```bash
# Check for security vulnerabilities
cargo audit
```

### Database Development

**Create a new migration**
```bash
sqlx migrate add <migration_name>
```

**Revert last migration**
```bash
sqlx migrate revert
```

### Debugging

1. **Enable debug logging**
   - Set `LOG_LEVEL=debug` in `.env`
   - Set `LOG_SQL_QUERIES=true` to see database queries

2. **Use mock SMS in development**
   - Set `SMS_PROVIDER=mock` to avoid sending real SMS
   - Verification codes will be printed to console

3. **Test with curl or Postman**
   - Import the Postman collection from `docs/postman/`
   - Or use the provided curl examples

## 📦 Project Structure

```
server/
├── api/                  # REST API server
│   ├── src/
│   │   ├── routes/       # API endpoints
│   │   ├── middleware/   # Auth, CORS, rate limiting
│   │   ├── dto/          # Request/response models
│   │   └── config.rs     # Configuration management
│   └── tests/            # API integration tests
├── core/                 # Business logic
│   └── src/
│       ├── domain/       # Entities and value objects
│       ├── services/     # Business services
│       └── repositories/ # Repository interfaces
├── infra/                # Infrastructure
│   └── src/
│       ├── database/     # MySQL implementations
│       ├── cache/        # Redis implementations
│       └── sms/          # SMS service
├── shared/               # Shared utilities
│   └── src/
│       ├── config/       # Configuration types
|       ├── errors/       # Common errors
│       ├── types/        # Common types
│       └── utils/        # Utility functions
└── migrations/           # Database migrations
```

## 🤝 Contributing

1. Create a feature branch: `git checkout -b feature/your-feature`
2. Make your changes and add tests
3. Ensure all tests pass: `cargo test`
4. Format your code: `cargo fmt`
5. Check code quality: `cargo clippy`
6. Commit with conventional commits: `feat(scope): description`
7. Push and create a pull request

## 🚀 Deployment

### Docker Deployment

The project includes comprehensive Docker support with different configurations for development and production environments.

#### Docker Files Overview

- **`Dockerfile`**: Multi-stage build for optimized Rust application
  - `builder` stage: Sets up build environment
  - `dependencies` stage: Caches Cargo dependencies
  - `build` stage: Compiles the application
  - `runtime` stage: Minimal production image
  - `development` stage: Includes development tools

- **`docker-compose.yml`**: Base configuration with MySQL, Redis, and backend services
- **`docker-compose.override.yml`**: Development overrides (auto-loaded in development)
- **`docker-compose.prod.yml`**: Production-specific configuration with security hardening

#### Development Deployment

```bash
# Start all services in development mode
docker-compose up -d

# With development tools (phpMyAdmin, RedisInsight)
docker-compose --profile dev-tools up -d

# Rebuild after code changes
docker-compose up -d --build backend

# View real-time logs
docker-compose logs -f backend

# Stop all services
docker-compose down

# Reset everything (including volumes)
docker-compose down -v
```

#### Production Deployment

```bash
# Build production image
docker build --target production -t renoveasy-backend:latest .

# Deploy with production configuration
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d

# Deploy with specific version
IMAGE_TAG=v1.0.0 docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d

# Enable automatic backups
docker-compose -f docker-compose.yml -f docker-compose.prod.yml --profile backup up -d

# Scale backend service
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d --scale backend=3
```

#### Container Management

```bash
# View running containers
docker-compose ps

# Execute commands in container
docker-compose exec backend cargo test
docker-compose exec mysql mysqldump -u root -p renoveasy > backup.sql

# Access container shell
docker-compose exec backend /bin/bash
docker-compose exec mysql mysql -u root -p

# Monitor resource usage
docker stats

# Clean up unused resources
docker system prune -a
```

### Environment-Specific Configuration

- **Development**: Use `.env.development` with mock services
  - Mock SMS provider
  - Simple passwords
  - Debug logging enabled
  - Hot reload support

- **Staging**: Use `.env.staging` with test databases
  - Real SMS provider (test account)
  - Moderate security settings
  - Info-level logging

- **Production**: Use `.env.production` with production services
  - Production SMS provider
  - Strong passwords and secrets
  - Security headers enabled
  - Optimized logging

⚠️ **Important**: Never commit `.env.production` or any file with real secrets!

## 🔒 Security Considerations

- All endpoints use HTTPS in production
- JWT tokens expire after 15 minutes (access) and 7 days (refresh)
- Phone numbers are hashed before storage
- Rate limiting prevents abuse (3 SMS per hour per phone)
- SQL injection protection via parameterized queries
- Input validation on all endpoints

## 📊 Monitoring

- **Health Check**: `GET /health`
- **Metrics**: Available on port 9090 when enabled
- **Logging**: Structured JSON logs in production
- **Error Tracking**: Integrate with Sentry (optional)

## 🐛 Troubleshooting

### Common Issues

1. **Database connection failed**
   - Verify MySQL is running: `docker ps`
   - Check DATABASE_URL in `.env`
   - Ensure database exists and user has permissions

2. **Redis connection failed**
   - Verify Redis is running: `docker ps`
   - Check REDIS_URL in `.env`
   - Test connection: `redis-cli ping`

3. **SMS not sending**
   - Check SMS_PROVIDER configuration
   - In development, use `mock` provider
   - Verify API keys for production providers

4. **Port already in use**
   - Change SERVER_PORT in `.env`
   - Or stop the conflicting service

## 📝 License

This project is proprietary software. All rights reserved.

## 🆘 Support

For issues and questions:
- Create an issue in the GitHub repository
- Contact the development team
- Check the [API documentation](./API.md)

---

Built with ❤️ using Rust 🦀
