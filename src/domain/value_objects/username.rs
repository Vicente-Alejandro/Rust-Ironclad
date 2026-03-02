use serde::{Deserialize, Serialize};
use crate::config::validators;
use crate::errors::DomainError;
use crate::shared::validator::validate_username;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Username(String);

impl Username {
    /// Smart Constructor: Falla si no cumple las reglas del dominio
    pub fn new(value: String) -> Result<Self, DomainError> {
        validate_username(&value)
            .map_err(|_| DomainError::Validation("Invalid username format".into()))?;
        Ok(Self(value))
    }

    /// Hidrata el Value Object desde una fuente confiable (ej. Base de Datos)
    pub fn from_trusted(value: String) -> Self {
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}