# Template Project - Rust Backend Framework

An attempt to professional backend framework built with Rust, designed for maximum performance and efficiency.

## 🚀 Features

### Core
- ⚡ **Actix-web** - Ultra-fast async web framework
- 🔄 **Tokio** - Complete async runtime
- 🗄️ **PostgreSQL** with SQLx (type-safe queries)
- 🍃 **MongoDB** - Optional NoSQL support
- 🔐 **JWT** - JSON Web Token authentication
- 🔒 **Bcrypt** - Secure password hashing

## 📁 Project Structure

```
project
├─ .env
├─ .env.example
├─ Cargo.lock
├─ Cargo.toml
├─ migrations
│  ├─ 001_create_users_table.sql
│  └─ 002_add_role_to_users.sql
├─ project.json
├─ README.md
└─ src
  ├─ config
  │  └─ mod.rs
  ├─ db
  │  ├─ mod.rs
  │  ├─ mongo.rs
  │  └─ postgres.rs
  ├─ errors
  │  └─ mod.rs
  ├─ handlers
  │  ├─ auth.rs
  │  ├─ examples
  │  │  └─ users.example.rs
  │  ├─ mod.rs
  │  └─ users.rs
  ├─ main.rs
  ├─ middleware
  │  ├─ auth.rs
  │  ├─ mod.rs
  │  └─ role.rs
  ├─ models
  │  ├─ mod.rs
  │  └─ user.rs
  ├─ routes
  │  └─ mod.rs
  └─ utils
    ├─ auth.rs
    ├─ jwt.rs
    └─ mod.rs
```

## 🔧 Quick Setup

### Requirements
- Rust 1.70+
- PostgreSQL 12+ (optional MongoDB)

### Steps

1. **Clone and configure .env:**
```bash
# Copy the .env file and adjust values
cp .env.example .env
```

2. **Create PostgreSQL database:**
```bash
createdb template_db
```

3. **Run migrations:**
```bash
# Use sqlx-cli
cargo install sqlx-cli
sqlx migrate run
```

4. **Build and run:**
```bash
cargo build --release
cargo run
```

Server will be available at `http://127.0.0.1:8080`

## 📚 API Endpoints

### Authentication

#### Register
```bash
POST /api/auth/register
Content-Type: application/json

{
  "email": "user@example.com",
  "username": "john_doe",
  "password": "SecurePassword123"
}
```

#### Login
```bash
POST /api/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "SecurePassword123"
}
```

**Response:**
```json
{
  "user": {
   "id": "550e8400-e29b-41d4-a716-446655440000",
   "email": "user@example.com",
   "username": "john_doe",
   "is_active": true,
   "created_at": "2025-12-27T10:30:00Z"
  },
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
}
```

### Users

#### Get Profile (Requires Authentication)
```bash
GET /api/users
Authorization: Bearer <token>
```

#### Delete Profile (Requires Authentication)
```bash
DELETE /api/users
Authorization: Bearer <token>
```

## 🔐 Security

### Implemented
- ✅ Passwords hashed with Bcrypt (DEFAULT_COST = 12)
- ✅ JWT with configurable expiration
- ✅ Input validation on all endpoints
- ✅ CORS ready (add when needed)

### Production Recommendations
1. Change `JWT_SECRET` to strong value
2. Use HTTPS in production
3. Implement rate limiting
4. Add CORS as needed
5. Monitoring and alerts

## 🗄️ Database

### PostgreSQL
SQLx provides:
- Type-safe queries (compiled at compile time)
- Automatic prepared statements
- Connection pooling

## 📦 Main Dependencies

| Library | Purpose | Reason |
|---------|---------|--------|
| actix-web | Web framework | Fastest, flexible and mature |
| tokio | Async runtime | Industry standard |
| sqlx | Type-safe ORM | Compile-time safety |
| mongodb | NoSQL | Optional flexibility |
| jsonwebtoken | JWT | Authentication standard |
| bcrypt | Password hashing | Secure & industry standard |
| validator | Validation | Derivable macros |
| tracing | Logging | Modern and structured |

## ❌ Not Included (on purpose)

- **Diesel** - More complex than SQLx, less flexible
- **Rocket** - Slower than Actix-web
- **SeaORM** - Not production-ready yet
- **Tests in v1** - Will be added in future versions
- **CORS/Rate Limit** - Add as needed

## 🚀 Future Improvements

- [ ] Unit and integration tests
- [ ] CORS middleware
- [ ] Rate limiting
- [ ] Refresh tokens
- [ ] Roles and permissions
- [ ] Soft delete users
- [ ] More CRUD endpoints
- [ ] WebSocket support
- [ ] GraphQL layer (optional)
- [ ] Caching with Redis

## 📝 Environment Variables

```env
SERVER_HOST=127.0.0.1          # Server host
SERVER_PORT=8080                # Server port
ENVIRONMENT=development          # development/staging/production

DATABASE_URL=...                # PostgreSQL URL
DB_MAX_CONNECTIONS=5            # Pool size

MONGODB_URL=...                 # MongoDB URL (optional)
MONGODB_NAME=template_db        # MongoDB database name

JWT_SECRET=...                  # JWT secret key
JWT_EXPIRATION=86400            # Seconds (default: 24h)
```

## 💡 Development Tips

### Fast compilation
```bash
cargo check  # Verify without building binary
```

### Optimized release
```bash
cargo build --release
```

### View detailed logs
```bash
RUST_LOG=debug cargo run
```

## 📄 License

MIT
