use serde::{Deserialize, Serialize};
use std::fmt;

/// Role value object - Enum para type safety
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Moderator,
    User,
    Premium,
}

impl Role {
    /// Get string representation of the role
    pub fn as_str(&self) -> &str {
        match self {
            Role::Admin => "admin",
            Role::Moderator => "moderator",
            Role::User => "user",
            Role::Premium => "premium",
        }
    }

    /// Crear Role desde string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "admin" => Role::Admin,
            "moderator" => Role::Moderator,
            "premium" => Role::Premium,
            _ => Role::User, // Default
        }
    }

    /// Verificar si es admin
    pub fn is_admin(&self) -> bool {
        matches!(self, Role::Admin)
    }

    /// Verificar si puede moderar
    pub fn can_moderate(&self) -> bool {
        matches!(self, Role::Admin | Role::Moderator)
    }

    /// Verificar si tiene acceso premium o superior
    pub fn is_premium_or_higher(&self) -> bool {
        matches!(self, Role::Admin | Role::Moderator | Role::Premium)
    }

    /// Check if it's a specific role
    pub fn has_role(&self, required_role: &str) -> bool {
        self.as_str() == required_role
    }

    /// Verificar si tiene cualquiera de los roles especificados
    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        roles.contains(&self.as_str())
    }

    /// Listar todos los roles disponibles
    pub fn all() -> Vec<Role> {
        vec![Role::Admin, Role::Moderator, Role::Premium, Role::User]
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for Role {
    fn default() -> Self {
        Role::User
    }
}

// Conversion for SQLx (String)
impl From<Role> for String {
    fn from(role: Role) -> String {
        role.as_str().to_string()
    }
}

impl From<&Role> for String {
    fn from(role: &Role) -> String {
        role.as_str().to_string()
    }
}

// Conversion from String
impl From<String> for Role {
    fn from(s: String) -> Self {
        Role::from_str(&s)
    }
}

impl From<&str> for Role {
    fn from(s: &str) -> Self {
        Role::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_admin() {
        assert!(Role::Admin.is_admin());
        assert!(!Role::User.is_admin());
    }

    #[test]
    fn test_can_moderate() {
        assert!(Role::Admin.can_moderate());
        assert!(Role::Moderator.can_moderate());
        assert!(!Role::User.can_moderate());
    }

    #[test]
    fn test_from_str() {
        assert_eq!(Role::from_str("admin"), Role::Admin);
        assert_eq!(Role::from_str("unknown"), Role::User);
    }

    #[test]
    fn test_has_any_role() {
        assert!(Role::Admin.has_any_role(&["admin", "moderator"]));
        assert!(!Role::User.has_any_role(&["admin", "moderator"]));
    }
}
