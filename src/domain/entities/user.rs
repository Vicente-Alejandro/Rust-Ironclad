use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::domain::value_objects::Role;

// Imports para SQLx
use sqlx::postgres::PgRow;
use sqlx::Row;

/// User domain entity - Pure business logic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub role: Role,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl sqlx::FromRow<'_, PgRow> for User {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let role_str: String = row.try_get("role")?;
        let role = Role::from_str(&role_str)  
            .ok_or_else(|| sqlx::Error::Decode(
                format!("Invalid role: {}", role_str).into()
            ))?;

        Ok(User {
            id: row.try_get("id")?,
            email: row.try_get("email")?,
            username: row.try_get("username")?,
            password_hash: row.try_get("password_hash")?,
            role,
            is_active: row.try_get("is_active")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

impl User {
    /// Crear nuevo usuario con valores por defecto
    pub fn new(email: String, username: String, password_hash: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            email,
            username,
            password_hash,
            role: Role::default(),
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create user with specific role
    pub fn new_with_role(
        email: String,
        username: String,
        password_hash: String,
        role: Role,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            email,
            username,
            password_hash,
            role,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    // ============================================
    // Business Logic Methods (Domain Logic)
    // ============================================

    /// Verificar si el usuario es admin
    pub fn is_admin(&self) -> bool {
        self.role.is_admin()
    }

    /// Verificar si puede moderar
    pub fn can_moderate(&self) -> bool {
        self.role.can_moderate()
    }

    /// Check if user is active
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    // ============================================
    // Mutation Methods (Update timestamp)
    // ============================================

    /// Actualizar email
    pub fn update_email(&mut self, email: String) {
        self.email = email;
        self.updated_at = Utc::now();
    }

    /// Actualizar username
    pub fn update_username(&mut self, username: String) {
        self.username = username;
        self.updated_at = Utc::now();
    }

    /// Actualizar password hash
    pub fn update_password_hash(&mut self, password_hash: String) {
        self.password_hash = password_hash;
        self.updated_at = Utc::now();
    }

    /// Cambiar role (solo admin puede hacerlo)
    pub fn change_role(&mut self, role: Role) {
        self.role = role;
        self.updated_at = Utc::now();
    }

    /// Activar usuario
    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }

    /// Desactivar usuario
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    // ============================================
    // Conversions
    // ============================================

    /// Convertir a DTO de respuesta (sin datos sensibles)
    pub fn to_response(&self) -> crate::application::dtos::UserResponse {
        use crate::application::dtos::UserResponse;

        UserResponse {
            id: self.id.clone(),
            email: self.email.clone(),
            username: self.username.clone(),
            role: self.role.to_string(),
            is_active: self.is_active,
            created_at: self.created_at.to_rfc3339(),
            updated_at: self.updated_at.to_rfc3339(),
        }
    }
}

/// JWT Claims - for authentication token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub role: String,
    pub exp: i64,
    pub iat: i64,
}

impl Claims {
    pub fn new(user_id: String, email: String, role: String, exp: i64) -> Self {
        let iat = Utc::now().timestamp();
        Self {
            sub: user_id,
            email,
            role,
            exp,
            iat,
        }
    }

    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }

    pub fn has_role(&self, required_role: &str) -> bool {
        self.role == required_role
    }

    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        roles.contains(&self.role.as_str())
    }
}