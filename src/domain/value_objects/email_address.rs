use serde::{Deserialize, Serialize};
use crate::errors::DomainError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAddress(String);

impl EmailAddress {
    /// Smart Constructor: Basic domain validation for the email
    pub fn new(value: String) -> Result<Self, DomainError> {
        if value.trim().is_empty() || !value.contains('@') {
            return Err(DomainError::Validation("Invalid email format at domain level".into()));
        }
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