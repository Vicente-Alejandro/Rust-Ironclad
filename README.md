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
- [📋 Dependencies](#-dependencies)

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
├─ Cargo.lock
├─ Cargo.toml
├─ LICENSE
├─ migrations
│  ├─ 001_create_users_table.sql
│  ├─ 002_add_role_to_users.sql
│  └─ 003_create_test_table.sql
├─ README.md
├─ src
│  ├─ application
│  │  ├─ dtos
│  │  │  ├─ auth_dto.rs
│  │  │  ├─ mod.rs
│  │  │  └─ test_item_dto.rs
│  │  ├─ mod.rs
│  │  └─ services
│  │     ├─ auth_service.rs
│  │     ├─ mod.rs
│  │     ├─ test_item_service.rs
│  │     └─ user_service.rs
│  ├─ cli
│  │  ├─ main.rs
│  │  └─ mod.rs
│  ├─ config
│  │  └─ mod.rs
│  ├─ db
│  │  ├─ mod.rs
│  │  ├─ mongo.rs
│  │  └─ postgres.rs
│  ├─ domain
│  │  ├─ entities
│  │  │  ├─ mod.rs
│  │  │  ├─ test_item.rs
│  │  │  └─ user.rs
│  │  ├─ mod.rs
│  │  └─ value_objects
│  │     ├─ mod.rs
│  │     └─ role.rs
│  ├─ errors
│  │  └─ mod.rs
│  ├─ infrastructure
│  │  ├─ http
│  │  │  ├─ authentication.rs
│  │  │  ├─ controllers
│  │  │  │  ├─ auth_controller.rs
│  │  │  │  ├─ health_controller.rs
│  │  │  │  ├─ mod.rs
│  │  │  │  ├─ test_item_controller.rs
│  │  │  │  └─ user_controller.rs
│  │  │  └─ mod.rs
│  │  ├─ mod.rs
│  │  └─ persistence
│  │     ├─ mod.rs
│  │     └─ postgres
│  │        ├─ mod.rs
│  │        ├─ test_item_repository.rs
│  │        └─ user_repository.rs
│  ├─ interfaces
│  │  ├─ mod.rs
│  │  └─ repositories
│  │     ├─ mod.rs
│  │     ├─ test_item_repository.rs
│  │     └─ user_repository.rs
│  ├─ main.rs
│  ├─ middleware
│  │  ├─ maintenance.rs
│  │  └─ mod.rs
│  ├─ routes
│  │  ├─ api.rs
│  │  └─ mod.rs
│  ├─ shared
│  │  ├─ extractors
│  │  │  ├─ mod.rs
│  │  │  └─ validated_json.rs
│  │  ├─ mod.rs
│  │  └─ validator
│  │     └─ mod.rs
│  ├─ storage
│  │  ├─ app
│  │  └─ framework
│  └─ utils
│     ├─ auth.rs
│     ├─ jwt.rs
│     └─ mod.rs
└─ storage
   └─ framework
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
cd ironclad
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
./target/release/ironclad
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

---

## 🖥️ CLI Commands

The framework includes a powerful CLI tool inspired by Laravel Artisan for common development tasks.

### Running the Application

The project has two binaries configured:

| Binary | Command | Purpose |
|--------|---------|---------|
| **`ironclad`** | `cargo run --bin main` | API Server (default) |
| **`ironclad`** | `cargo run --bin ironclad` | CLI Tool |

> **Note:** You can change the default binary in `Cargo.toml` under `[package]` → `default-run`

---

### 📋 Available Commands

#### **System Information**
```bash
# Show framework version and info
cargo run --bin ironclad -- version
```

**Output:**
```
╔════════════════════════════════════════╗
║   🦀 Rust Ironclad Framework v0.0.3   ║
╚════════════════════════════════════════╝

Framework: Rust Ironclad
Version: 0.0.3
```

---

#### **Database Operations**
```bash
# Check database connection and health
cargo run --bin ironclad -- db-check
```

**Output:**
```
🔍 Checking database connection...
📍 Database: postgresql://postgres:****@localhost:5432/rust

🔌 Connecting... ✅
📡 Sending ping... ✅

╔═══════════════════════════════╗
║  ✅ Database is UP and ready  ║
╚═══════════════════════════════╝
```

---

#### **Maintenance Mode** (Laravel-style)

Put your application into maintenance mode to safely perform updates:
```bash
# Enable maintenance mode
cargo run --bin ironclad -- down

# Custom retry time (in seconds)
cargo run --bin ironclad -- down --retry 30

# With custom message
cargo run --bin ironclad -- down --message "Database migration in progress"

# With custom retry 
cargo run --bin ironclad -- down --message "Scheduled maintenance" --retry 300
```

**While in maintenance mode, all API requests return:**
```json
{
  "error": "Service Unavailable",
  "message": "Database migration in progress",
  "status": 503,
  "retry_after": 60
}
```

**Bring the application back online:**
```bash
# Disable maintenance mode
cargo run --bin ironclad -- up
```

---

#### **Diagnostics**
```bash
# Run CLI system checks
cargo run --bin ironclad -- test
```

---

### 🚀 Quick Reference

<table>
<tr>
<th>Task</th>
<th>Command</th>
</tr>

<tr>
<td>Start API Server</td>
<td>
```bash
cargo run --bin main
```

</td>
</tr>

<tr>
<td>Show CLI Help</td>
<td>
```bash
cargo run --bin ironclad -- --help
```

</td>
</tr>

<tr>
<td>Check Database</td>
<td>
```bash
cargo run --bin ironclad -- db-check
```

</td>
</tr>

<tr>
<td>Enable Maintenance</td>
<td>
```bash
cargo run --bin ironclad -- down
```

</td>
</tr>

<tr>
<td>Disable Maintenance</td>
<td>
```bash
cargo run --bin ironclad -- up
```

</td>
</tr>

<tr>
<td>Version Info</td>
<td>
```bash
cargo run --bin ironclad -- version
```

</td>
</tr>

</table>

---

### 💡Tips

**Create an alias for faster development:**

**Bash/Zsh (Linux/macOS):**
```bash
# Add to ~/.bashrc or ~/.zshrc
alias ironclad='cargo run --bin ironclad --'

# Usage
ironclad version
ironclad db-check
ironclad down --message "Updating..."
ironclad -- down --retry 30
```

**PowerShell (Windows):**
```powershell
# Add to your PowerShell profile
function ironclad { cargo run --bin ironclad -- $args }

# Usage
ironclad version
ironclad db-check
ironclad down --message "Updating..."
ironclad -- down --retry 30 
```

**Install globally for production:**
```bash
cargo install --path . --bin ironclad

# Now use directly
ironclad version
ironclad up
ironclad down
```

---

### 🛠️ Development Workflow
```bash
# 1. Start the server
cargo run --bin main

# 2. In another terminal, check database
cargo run --bin ironclad -- db-check

# 3. Put in maintenance mode for updates
cargo run --bin ironclad -- down --message "Deploying new features"

# 4. Run migrations, updates, etc.
sqlx migrate run

# 5. Bring back online
cargo run --bin ironclad -- up
```

---

## 🔧 Maintenance Mode (Laravel-style)

Put your application into maintenance mode to safely perform updates, migrations, or deployments.

### What is Maintenance Mode?

When you activate maintenance mode:
- ✅ Your server keeps running
- ⛔ All requests receive a **503 Service Unavailable** response
- 🎨 Users see a custom page (HTML) or JSON message
- 🔑 You can create a secret bypass (you and your team can keep working)

---

### Basic Commands

#### **Activate Maintenance Mode**
```bash
# Basic (default message)
cargo run --bin ironclad -- down

# With custom message
cargo run --bin ironclad -- down --message "Updating database..."

# With custom retry time (in seconds)
cargo run --bin ironclad -- down --message "Scheduled maintenance" --retry 300
```

#### **Deactivate Maintenance Mode**
```bash
cargo run --bin ironclad -- up
```

---

### 🔑 Secret Bypass (Team Access)

Allows your team to keep accessing the application while it's in maintenance.
```bash
# Activate with secret
cargo run --bin ironclad -- down --secret "myteam2024"
```

**How it works:**

1. A regular user tries to access:
```
   http://localhost:8080/api/users
   → 503 Maintenance (blocked ❌)
```

2. Your team accesses with the secret:
```
   http://localhost:8080/api/users/myteam2024
   → Redirects to /api/users
   → Bypass cookie is saved
   → 200 OK (full access ✅)
```

3. After the first access, the secret is no longer needed (cookie saved):
```
   http://localhost:8080/api/users
   → 200 OK (active cookie ✅)
```

---

### 🎨 Custom HTML Templates

Display maintenance pages with custom design when users access from a browser.

#### **Template Structure**
```
src/
storage/
templates/
└── render/
    └── down/
        ├── default.html              # Default template
        ├── emergency/
        │   ├── default.html          # --render "emergency"
        │   └── low.html              # --render "emergency::low"
        └── maintenance/
            └── database.html         # --render "maintenance::database"
```

#### **Using Templates**
```bash
# Without specifying (uses default.html)
cargo run --bin ironclad -- down

# Use specific folder (loads its default.html)
cargo run --bin ironclad -- down --render "emergency"
# → Loads: templates/render/down/emergency/default.html

# Use specific file
cargo run --bin ironclad -- down --render "emergency::low"
# → Loads: templates/render/down/emergency/low.html

# Database template
cargo run --bin ironclad -- down --render "maintenance::database" --message "Migrating to PostgreSQL 16"
# → Loads: templates/render/down/maintenance/database.html
```

#### **Create Your Own Template**

1. **Create folder and file:**
```bash
   mkdir templates\render\down\myfolder
   # Create file: templates\render\down\myfolder\default.html
```

2. **HTML content:**
```html
   <!DOCTYPE html>
   <html>
   <head>
       <title>Maintenance</title>
       <style>
           body { 
               background: #2c3e50; 
               color: white; 
               text-align: center; 
               padding-top: 100px;
           }
       </style>
   </head>
   <body>
       <h1>🚧 Under Maintenance</h1>
       <p>{{MESSAGE}}</p>
       <p>Come back in {{RETRY}} seconds</p>
   </body>
   </html>
```

3. **Use your template:**
```bash
   cargo run --bin ironclad -- down --render "myfolder"
```

**Available variables:**
- `{{MESSAGE}}` - Your custom message
- `{{RETRY}}` - Retry time in seconds
- `{{TIMESTAMP}}` - Activation date/time

---

### 📋 JSON Mode (no HTML)

Forces JSON responses even for browsers (useful for pure APIs).
```bash
cargo run --bin ironclad -- down --norender
```

**Response:**
```json
{
  "error": "Service Unavailable",
  "message": "Application is down for maintenance",
  "status": 503,
  "retry_after": 60
}
```

---

### ↪️ Redirect

Redirects all requests to a specific URL (useful for an external status page).
```bash
# Redirect to internal route
cargo run --bin ironclad -- down --redirect "/status"

# Redirect to external site
cargo run --bin ironclad -- down --redirect "https://status.myapp.com"
```

---

### 📊 Full Examples

#### **Case 1: Scheduled Maintenance**
```bash
# Activate
cargo run --bin ironclad -- down \
  --message "Scheduled maintenance: security update" \
  --retry 1800 \
  --secret "admin2024"

# While you work, access with:
# http://localhost:8080/api/any-route/admin2024

# When done
cargo run --bin ironclad -- up
```

#### **Case 2: Emergency (Team Only)**
```bash
cargo run --bin ironclad -- down \
  --render "emergency" \
  --message "Security incident detected. Resolving..." \
  --secret "emergency-access" \
  --norender  # Forces JSON for APIs
```

#### **Case 3: Database Migration**
```bash
# Step 1: Activate maintenance
cargo run --bin ironclad -- down \
  --render "maintenance::database" \
  --message "Migrating database from MySQL to PostgreSQL" \
  --retry 600 \
  --secret "dbteam"

# Step 2: Your team accesses with /dbteam at the end of any URL
# http://localhost:8080/api/users/dbteam

# Step 3: Run migration
sqlx migrate run

# Step 4: Deactivate
cargo run --bin ironclad -- up
```

#### **Case 4: Deployment with External Status Page**
```bash
cargo run --bin ironclad -- down \
  --redirect "https://status.myapp.com/deploy-in-progress" \
  --secret "deploy2024"
```

---

### 🔍 Check Status
```bash
# Check if maintenance file exists
dir storage\framework\maintenance.json

# View contents
Get-Content storage\framework\maintenance.json | ConvertFrom-Json

# Example output:
# time          : 1708550400
# message       : Updating database...
# retry         : 60
# created_at    : 2026-02-21T20:00:00Z
# secret        : myteam2024
# render        : emergency::low
```

---

### 🎯 Comparison with Laravel

| Laravel | Rust Ironclad | Description |
|---------|---------------|-------------|
| `php artisan down` | `cargo run --bin ironclad -- down` | Activate maintenance |
| `php artisan up` | `cargo run --bin ironclad -- up` | Deactivate maintenance |
| `--secret="token"` | `--secret "token"` | Bypass access |
| `--render="view"` | `--render "template"` | Custom view |
| `--redirect="/url"` | `--redirect "/url"` | Redirect |
| `--retry=600` | `--retry 600` | Retry time |

---

### 💡 Tips and Best Practices

1. **Store the secret securely:** Don't share it in public repositories
```bash
   # Good: Environment variable
   $SECRET = $env:MAINTENANCE_SECRET
   cargo run --bin ironclad -- down --secret $SECRET
```

2. **Use descriptive templates:** Create specific templates for each type of maintenance
```
   templates/render/down/
   ├── scheduled/     # Scheduled maintenance
   ├── emergency/     # Emergencies
   ├── deploy/        # Deployments
   └── database/      # DB Migrations
```

3. **Combine options as needed:**
```bash
   # API + Web with secret
   cargo run --bin ironclad -- down \
     --render "deploy" \
     --message "Deploying v2.0" \
     --secret "team" \
     --retry 300
```

4. **Automate with scripts:**
```bash
   # deploy.sh
   cargo run --bin ironclad -- down --secret "deploy-$(date +%s)"
   # ... deploy commands ...
   cargo run --bin ironclad -- up
```

---

### ⚠️ Important

- ❌ **DO NOT use `--render` and `--redirect` together** (conflict)
- ❌ **DO NOT use `--norender` and `--render` together** (conflict)
- ✅ **Always run from the project root** (where `Cargo.toml` is)
- ✅ **Templates must be in `templates/`, NOT in `src/templates/`**

---

### 🐛 Troubleshooting

**Problem:** "Template not found"
```bash
# Solution: Verify location
tree /F templates
# Templates must be in templates/, not in src/templates/
```

**Problem:** "Application is not in maintenance mode" when running `up`
```bash
# Solution: Check file
dir storage\framework\maintenance.json
# If it doesn't exist, the app is not in maintenance mode
```

**Problem:** The server keeps responding normally
```bash
# Solution 1: Restart the server
# Ctrl+C and run again: cargo run

# Solution 2: Verify the file was created
Get-Content storage\framework\maintenance.json
```

**Problem:** Secret is not working
```bash
# Solution: Clear browser cookies
# Or use incognito mode to test
```

---

### 📁 Related Files
```
project/
├── storage/
│   └── framework/
│       └── maintenance.json      # Maintenance state (auto-generated)
├── templates/
│   └── render/
│       └── down/                 # HTML Templates
└── src/
    ├── cli/main.rs               # down/up commands
    └── middleware/maintenance.rs # Maintenance logic
```

---

Questions? Create an [issue](https://github.com/Vicente-Alejandro/Rust-Ironclad/issues) or [PR](https://github.com/Vicente-Alejandro/Rust-Ironclad/pulls)

</div>
