use sqlx::PgPool;
use crate::errors::ApiError;
use crate::monitoring::alerts::{Alert, AlertLevel};
use crate::application::services::AlertService;

pub struct QueueMonitor {
    pool: PgPool,
}

impl QueueMonitor {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn check_alerts(&self) -> Result<(), ApiError> {

        // 1. Failed jobs
        let failed: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM job_queue WHERE status = 'failed'"
        )
        .fetch_one(&self.pool)
        .await?;

        if failed.0 > 10 {
            AlertService::send(Alert {
                level: AlertLevel::Critical,
                message: "High number of failed jobs".into(),
                metadata: serde_json::json!({ "failed_jobs": failed.0 }),
            }).await;
        }

        // 2. Dead letter queue
        let dead: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM dead_letter_queue"
        )
        .fetch_one(&self.pool)
        .await?;

        if dead.0 > 5 {
            AlertService::send(Alert {
                level: AlertLevel::Warning,
                message: "Dead letter queue growing".into(),
                metadata: serde_json::json!({ "dead_jobs": dead.0 }),
            }).await;
        }

        // 3. Pending backlog
        let pending: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM job_queue WHERE status = 'pending'"
        )
        .fetch_one(&self.pool)
        .await?;

        if pending.0 > 50 {
            AlertService::send(Alert {
                level: AlertLevel::Warning,
                message: "Queue backlog is high".into(),
                metadata: serde_json::json!({ "pending_jobs": pending.0 }),
            }).await;
        }

        Ok(())
    }
}