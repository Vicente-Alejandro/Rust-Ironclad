# ⚙️ Rust Ironclad Framework

> Enterprise-grade backend framework built with Rust for maximum performance and scalability.

[![Rust](https://img.shields.io/badge/Rust-1.70+-CE422B?logo=rust&style=for-the-badge)](https://www.rust-lang.org/)
[![Actix-web](https://img.shields.io/badge/Actix--web-4.4-00A500?style=for-the-badge)](https://actix.rs/)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-12+-336791?logo=postgresql&style=for-the-badge)](https://www.postgresql.org/)
[![License](https://img.shields.io/badge/License-MIT-yellow?style=for-the-badge)](LICENSE)
[![Status](https://img.shields.io/badge/Status-Active-00FF00?style=for-the-badge)]()

**Last Updated:** `v0.0.3` • `15-02-26`

---

## 📖 Table of Contents

- [✨ Key Features](#-key-features)
- [🏗️ Architecture](#-architecture)
- [📦 Project Structure](#-project-structure)
- [🚀 Quick Start](#-quick-start)
- [📚 API Endpoints](#-api-endpoints)
- [🔐 Security](#-security)
- [📋 Dependencies](#-dependencies)
- [🛣️ Roadmap](#-roadmap)

---

## ✨ Key Features

<table>
<tr>
<td width="50%">

**Performance**
- ⚡ Ultra-fast Actix-web framework (50k+ req/s)
- 🔄 Non-blocking async runtime (Tokio)
- 🚀 Zero-copy response streaming
- 💾 Connection pooling

</td>
<td width="50%">

**Security**
- 🔐 JWT stateless authentication
- 🔒 Bcrypt password hashing (cost 12)
- ✅ Input validation on all endpoints
- 🛡️ CORS ready

</td>
</tr>
<tr>
<td width="50%">

**Quality**
- 📝 Type-safe SQLx queries (compile-time)
- 🏛️ Clean Architecture (DDD pattern)
- 🧩 Modular & extensible design
- 🧪 Interface-based testing

</td>
<td width="50%">

**Data**
- 🗄️ PostgreSQL with SQLx
- 🍃 MongoDB support (optional)
- 📊 Pagination support
- 🔄 Connection pooling

</td>
</tr>
</table>

---

## 🏗️ Architecture

This framework follows **Domain-Driven Design (DDD)** with a clean 5-layer architecture:

```
┌─────────────────────────────────────┐
│  Routes Layer                       │ ← HTTP Routing
├─────────────────────────────────────┤
│  Infrastructure Layer               │ ← HTTP, Extractors, Controllers
├─────────────────────────────────────┤
│  Application Layer                  │ ← Services, DTOs, Use Cases
├─────────────────────────────────────┤
│  Domain Layer                       │ ← Entities, Value Objects, Business Logic
├─────────────────────────────────────┤
│  Interfaces Layer                   │ ← Trait Definitions (Repository Pattern)
└─────────────────────────────────────┘
```

**Key Design Patterns:**
- Repository Pattern (abstraction over data access)
- Dependency Injection (Inversion of Control)
- Value Objects (type-safe validation)
- Extractors (Actix authentication/role-based access)

---

## 📦 Project Structure

```
src/
├─ domain/                          # Pure Business Logic (DDD Core)
│  ├─ entities/
│  │  └─ user.rs                   # User entity with business rules
│  └─ value_objects/
│     └─ role.rs                   # Role enum with validation
│
├─ application/                     # Use Cases & Services
│  ├─ dtos/                        # Data Transfer Objects
│  │  └─ auth_dto.rs
│  └─ services/
│     ├─ auth_service.rs          # Authentication logic
│     └─ user_service.rs          # User management logic
│
├─ infrastructure/                 # Technical Implementation
│  ├─ http/
│  │  ├─ authentication.rs        # JWT extractors & role guards
│  │  ├─ controllers/              # HTTP handlers
│  │  │  ├─ auth_controller.rs
│  │  │  └─ user_controller.rs
│  │  └─ mod.rs
│  └─ persistence/
│     └─ postgres/
│        └─ user_repository.rs    # SQL queries & repository impl
│
├─ interfaces/                     # Trait Contracts
│  └─ repositories/
│     └─ user_repository.rs       # Repository interface
│
├─ shared/                         # Cross-cutting Utilities
│  ├─ extractors/                # Custom Actix extractors
│  └─ validator/                 # Input validation
│
├─ routes/                        # Route Configuration
│  └─ api.rs                     # Endpoint definitions
│
├─ config/                        # Configuration Management
├─ errors/                        # Error Handling
├─ db/                           # Database Initialization
├─ utils/                        # JWT, Auth Utilities
└─ main.rs                       # Server Entry Point
```

---

## 🚀 Quick Start

### Prerequisites
- **Rust** 1.70+ ([install](https://rustup.rs/))
- **PostgreSQL** 12+ ([install](https://www.postgresql.org/download/))
- **sqlx-cli** (for migrations)

### Setup Steps

#### 1️⃣ Clone & Configure
```bash
git clone <repository>
cd template_project
cp .env.example .env
# Edit .env with your database credentials
```

#### 2️⃣ Create Database
```bash
createdb template_db
```

#### 3️⃣ Run Migrations
```bash
cargo install sqlx-cli
sqlx migrate run
```

#### 4️⃣ Run Server
```bash
# Development
cargo run

# Release (optimized)
cargo build --release
./target/release/template_project
```

✅ Server running at `http://127.0.0.1:8080`

---

## 📚 API Endpoints

### 🔑 Authentication

#### Register User
```http
POST /api/auth/register
Content-Type: application/json

{
  "email": "user@example.com",
  "username": "john_doe",
  "password": "SecurePassword123"
}
```

**Response (201 Created):**
```json
{
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "username": "john_doe",
    "role": "User",
    "created_at": "2025-02-15T10:30:00Z"
  },
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
}
```

#### Login
```http
POST /api/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "SecurePassword123"
}
```

### 👤 Users

#### Get Profile (Authenticated)
```http
GET /api/user/profile
Authorization: Bearer {token}
```

#### List All Users (Public)
```http
GET /api/user/all/nosession
```

#### Get User by ID
```http
GET /api/user/{id}
```

---

## 🔐 Security

## 🔐 Security

### ✅ Implemented
| Feature | Details |
|---------|---------|
| **Password Hashing** | Bcrypt with cost factor 12 |
| **JWT Authentication** | Configurable expiration (default 24h) |
| **Input Validation** | Automatic validation on all endpoints |
| **Type Safety** | Compile-time query validation with SQLx |
| **Role-Based Access** | Extractors for Admin, Moderator, Premium roles |

### 🔧 Production Checklist
- [ ] Change `JWT_SECRET` in `.env`
- [ ] Enable HTTPS/TLS
- [ ] Implement rate limiting
- [ ] Add request logging & monitoring
- [ ] Set `ENVIRONMENT=production`
- [ ] Enable database backups
- [ ] Configure CORS as needed

---

## 📋 Dependencies

| Package | Version | Purpose | Why? |
|---------|---------|---------|------|
| **actix-web** | 4.4 | Web framework | Fastest (50k+ req/s), flexible, mature |
| **tokio** | 1.35 | Async runtime | Industry standard, production-ready |
| **sqlx** | 0.7 | Type-safe ORM | Compile-time safety, zero runtime overhead |
| **serde** | 1.0 | Serialization | JSON serialization/deserialization |
| **jsonwebtoken** | 9.2 | JWT auth | Standard, proven, battle-tested |
| **bcrypt** | 0.15 | Password hashing | Slow-by-design, industry standard |
| **async-trait** | 0.1 | Async traits | Required for async repository pattern |
| **tracing** | 0.1 | Structured logging | Modern, async-aware, high-performance |
| **actix-cors** | 0.7 | CORS middleware | Built for Actix, easy configuration |

[See full Cargo.toml](./Cargo.toml)

---

## 💡 Development

### Common Commands
```bash
# Check compilation without building
cargo check

# Run with debug logs
RUST_LOG=debug cargo run

# Format code
cargo fmt

# Run clippy linter
cargo clippy

# Build optimized release
cargo build --release

# Run tests
cargo test
```

### Environment Variables
```env
# Server
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
ENVIRONMENT=development

# Database
DATABASE_URL=postgresql://user:password@localhost/template_db
DB_MAX_CONNECTIONS=5

# MongoDB (optional)
MONGODB_URL=mongodb://localhost:27017
MONGODB_NAME=template_db

# JWT
JWT_SECRET=your_secret_key_here
JWT_EXPIRATION=86400
```

---

## 🛣️ Roadmap

| Phase | Features | Status |
|-------|----------|--------|
| **v0.1** | Core DDD architecture, JWT auth, CRUD users | ✅ Done |
| **v0.2** | Role-based access control, pagination, validation | ✅ Done |
| **v0.3** | Routes module, controllers separation | ✅ Done |
| **v0.4** | Integration tests, API docs (OpenAPI) | 🔄 In Progress |
| **v0.5** | Rate limiting, caching (Redis), audit logging | 📋 Planned |
| **v1.0** | Production-ready, monitoring, CI/CD | 📋 Planned |

---

## 📂 Project Config

### .env.example
```env
# Server Configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
ENVIRONMENT=development

# PostgreSQL
DATABASE_URL=postgresql://postgres:password@localhost/template_db
DB_MAX_CONNECTIONS=10

# JWT
JWT_SECRET=your_super_secret_key_change_this_in_production
JWT_EXPIRATION=86400

# MongoDB (optional, leave empty to skip)
MONGODB_URL=
MONGODB_NAME=template_db
```

---

## 🤝 Best Practices

### Code Organization
- **Domain** = Pure business logic (no dependencies on framework)
- **Application** = Orchestration of business logic
- **Infrastructure** = Framework & database details
- **Interfaces** = Trait definitions (testable with mocks)

## � Learning Resources

- [Actix-web Documentation](https://actix.rs/)
- [Tokio Async Runtime](https://tokio.rs/)
- [SQLx Type-Safe SQL](https://github.com/launchbadge/sqlx)
- [Domain-Driven Design by Eric Evans](https://www.domainlanguage.com/ddd/)
- [Clean Architecture by Robert C. Martin](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)

---

## ⭐ Support

If this framework helps you, consider giving it a star! ⭐

---

## 📄 License

MIT License - Feel free to use in your projects

```
Copyright (c) 2026 Rust Ironclad Framework

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:
...
```

---
To create new object
src/
├─ domain/entities/
│  └─ test_item.rs          # 🆕 Entidad
├─ application/
│  ├─ dtos/
│  │  └─ test_item_dto.rs   # 🆕 DTOs
│  └─ services/
│     └─ test_item_service.rs # 🆕 Lógica de negocio
├─ infrastructure/
│  ├─ http/controllers/
│  │  └─ test_item_controller.rs # 🆕 HTTP handlers
│  └─ persistence/postgres/
│     └─ test_item_repository.rs # 🆕 Queries SQL
├─ interfaces/repositories/
│  └─ test_item_repository.rs # 🆕 Trait
└─ migrations/
   └─ 00X_create_test_items_table.sql # 🆕 Schema
---

+++
To use CLI commands:

cargo run --bin ironclad -- $arg

Example:
cargo run --bin ironclad -- test
+++

<div align="center">

### Made with ❤️ in Rust

Questions? Create an [issue](https://github.com/Vicente-Alejandro/Rust-Ironclad/issues) or [PR](https://github.com/Vicente-Alejandro/Rust-Ironclad/pulls)

</div>

