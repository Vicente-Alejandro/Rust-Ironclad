use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::postgres::PgRow;
use sqlx::Row;

/// TestItem domain entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestItem {
    pub id: String,
    pub subject: String,
    pub optional_field: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TestItem {
    /// Create new test item
    pub fn new(subject: String, optional_field: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            subject,
            optional_field,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update subject
    pub fn update_subject(&mut self, subject: String) {
        self.subject = subject;
        self.updated_at = Utc::now();
    }

    /// Update optional field
    pub fn update_optional_field(&mut self, optional_field: Option<String>) {
        self.optional_field = optional_field;
        self.updated_at = Utc::now();
    }

    /// Convert to response DTO
    pub fn to_response(&self) -> crate::application::dtos::test_item_dto::TestItemResponse {
        crate::application::dtos::test_item_dto::TestItemResponse {
            id: self.id.clone(),
            subject: self.subject.clone(),
            optional_field: self.optional_field.clone(),
            created_at: self.created_at.to_rfc3339(),
            updated_at: self.updated_at.to_rfc3339(),
        }
    }
}

// SQLx FromRow implementation
impl sqlx::FromRow<'_, PgRow> for TestItem {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(TestItem {
            id: row.try_get("id")?,
            subject: row.try_get("subject")?,
            optional_field: row.try_get("optional_field")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}