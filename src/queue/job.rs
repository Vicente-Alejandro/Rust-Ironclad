use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobType {
    DeleteTestItem,
    SendEmail,
    ProcessPayment,
    GenerateReport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum JobPayload {
    DeleteTestItem { item_id: String },
    SendEmail { to: String, subject: String, body: String },
    ProcessPayment { amount: f64, user_id: String },
    GenerateReport { report_type: String, user_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Job {
    pub id: String,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub priority: i32,

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