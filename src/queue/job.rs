use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]

// This enum defines the different types of jobs that can be processed by the worker. Each variant can have its own associated data as needed. For this example, we only implement a single job type (DeleteTestItem) to demonstrate the structure
pub enum JobType {
    DeleteTestItem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]

// Here would be the payload for different job types. For simplicity, we only implement DeleteTestItem in this example.
pub enum JobPayload {
    DeleteTestItem { item_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Job {
    pub id: String,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub priority: i32,
    pub queue_name: String,

    pub scheduled_at: DateTime<Utc>,
    pub retry_at: Option<DateTime<Utc>>,              
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,

    pub lock_expires_at: Option<DateTime<Utc>>,       
    pub worker_id: Option<String>,                    

    pub attempts: i32,
    pub max_attempts: i32,

    pub error_message: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}