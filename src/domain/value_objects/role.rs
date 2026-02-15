use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    Admin,
    User,
    Moderator,
    Premium,
}

impl Role {
    pub fn as_str(&self) -> &str {
        match self {
            Role::Admin => "admin",
            Role::User => "user",
            Role::Moderator => "moderator",
            Role::Premium => "premium",
        }
    }

    /// Parse role from string - returns Option for safety
    pub fn from_str(s: &str) -> Option<Self> {  // ✅ Cambiar a Option
        match s.to_lowercase().as_str() {
            "admin" => Some(Role::Admin),
            "user" => Some(Role::User),
            "moderator" => Some(Role::Moderator),
            "premium" => Some(Role::Premium),
            _ => None,  // ✅ Return None for invalid values
        }
    }

    pub fn is_admin(&self) -> bool {
        matches!(self, Role::Admin)
    }

    pub fn can_moderate(&self) -> bool {
        matches!(self, Role::Admin | Role::Moderator)
    }
}

impl Default for Role {
    fn default() -> Self {
        Role::User
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<Role> for String {
    fn from(role: Role) -> String {
        role.as_str().to_string()
    }
}