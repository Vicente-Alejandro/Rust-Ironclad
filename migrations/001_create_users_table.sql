-- Migración: crear tabla users
CREATE TABLE IF NOT EXISTS users (
    id VARCHAR(36) PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    username VARCHAR(100) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL
);

-- Indices para búsquedas rápidas
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);

-- Comentarios de descripción
COMMENT ON TABLE users IS 'Tabla principal de usuarios del sistema';
COMMENT ON COLUMN users.id IS 'ID único del usuario en formato UUID';
COMMENT ON COLUMN users.email IS 'Email único del usuario';
COMMENT ON COLUMN users.password_hash IS 'Contraseña hasheada con bcrypt';
COMMENT ON COLUMN users.is_active IS 'Indica si el usuario está activo o no';

