// CAN YOU TRANSLATE ALL OF THIS TO ENGLISH PLEASE

# Template Project - Framework Backend en Rust

Un framework backend profesional y producción-listo construido con Rust, diseñado para máximo rendimiento y eficiencia.

## 🚀 Características

### Core
- ⚡ **Actix-web** - Framework web async ultrarrápido
- 🔄 **Tokio** - Runtime async completo
- 🗄️ **PostgreSQL** con SQLx (queries type-safe)
- 🍃 **MongoDB** - Soporte opcional para NoSQL
- 🔐 **JWT** - Autenticación con JSON Web Tokens
- 🔒 **Bcrypt** - Hashing seguro de contraseñas

## 📁 Estructura del Proyecto

```
poroject
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
   │  │  └─ users.examle.rs
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

## 🔧 Configuración Rápida

### Requisitos
- Rust 1.70+
- PostgreSQL 12+ (opcional MongoDB)

### Pasos

1. **Clonar y configurar .env:**
```bash
# Copiar el archivo .env y ajustar valores
cp .env.example .env
```

2. **Crear base de datos PostgreSQL:**
```bash
createdb template_db
```

3. **Ejecutar migraciones:**
```bash
# Usar sqlx-cli
cargo install sqlx-cli
sqlx migrate run
```

4. **Compilar y ejecutar:**
```bash
cargo build --release
cargo run
```

El servidor estará disponible en `http://127.0.0.1:8080`

## 📚 Endpoints API

### Autenticación

#### Registro
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

### Usuarios

#### Obtener Perfil (Requiere Autenticación)
```bash
GET /api/users
Authorization: Bearer <token>
```

#### Eliminar Perfil (Requiere Autenticación)
```bash
DELETE /api/users
Authorization: Bearer <token>
```

## 🔐 Seguridad

### Implementado
- ✅ Contraseñas hasheadas con Bcrypt (DEFAULT_COST = 12)
- ✅ JWT con expiración configurable
- ✅ Validación de entrada en todos los endpoints
- ✅ CORS ready (agregar cuando sea necesario)

### Recomendaciones Producción
1. Cambiar `JWT_SECRET` a valor fuerte
2. Usar HTTPS en producción
3. Implementar rate limiting
4. Agregar CORS según necesidad
5. Monitoreo y alertas

## 🗄️ Base de Datos

### PostgreSQL
SQLx proporciona:
- Type-safe queries (compiladas en tiempo de compilación)
- Prepared statements automáticas
- Pool de conexiones

## 📦 Dependencias Principales

| Librería | Propósito | Razón |
|----------|-----------|-------|
| actix-web | Framework web | Más rápido, flexible y maduro |
| tokio | Runtime async | Estándar de la industria |
| sqlx | ORM type-safe | Seguridad en tiempo de compilación |
| mongodb | NoSQL | Flexibilidad opcional |
| jsonwebtoken | JWT | Standard de autenticación |
| bcrypt | Password hashing | Secure & industry standard |
| validator | Validación | Macros derivables |
| tracing | Logging | Moderno y estructurado |

## ❌ No Incluido (a propósito)

- **Diesel** - Más complejo que SQLx, menos flexible
- **Rocket** - Más lento que Actix-web
- **SeaORM** - Aún no listo para producción
- **Tests en v1** - Se agregará en próximas versiones
- **CORS/Rate Limit** - Agregar según necesidad

## 🚀 Próximas Mejoras

- [ ] Tests unitarios e integración
- [ ] CORS middleware
- [ ] Rate limiting
- [ ] Refresh tokens
- [ ] Roles y permisos
- [ ] Soft delete de usuarios
- [ ] Más endpoints CRUD
- [ ] WebSocket support
- [ ] GraphQL layer (opcional)
- [ ] Caching con Redis

## 📝 Variables de Entorno

```env
SERVER_HOST=127.0.0.1          # Host del servidor
SERVER_PORT=8080                # Puerto del servidor
ENVIRONMENT=development          # development/staging/production

DATABASE_URL=...                # URL de PostgreSQL
DB_MAX_CONNECTIONS=5            # Pool size

MONGODB_URL=...                 # URL de MongoDB (opcional)
MONGODB_NAME=template_db        # Nombre de BD MongoDB

JWT_SECRET=...                  # Clave secreta JWT
JWT_EXPIRATION=86400            # Segundos (default: 24h)
```

## 💡 Tips de Desarrollo

### Compilación rápida
```bash
cargo check  # Verificar sin compilar binario
```

### Release optimizado
```bash
cargo build --release
```

### Ver logs detallados
```bash
RUST_LOG=debug cargo run
```

## 📄 Licencia

MIT