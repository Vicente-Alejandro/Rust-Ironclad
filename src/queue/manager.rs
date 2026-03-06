use sqlx::{PgPool, Row};
use uuid::Uuid;  // Sigue siendo necesario para generar IDs
use chrono::{DateTime, Utc, Duration};
use serde_json;

use crate::errors::ApiError;
use super::job::{Job, JobPayload};

pub struct QueueManager {
    pool: PgPool,
}

impl QueueManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Enqueue a job to be executed immediately
    pub async fn enqueue(&self, payload: JobPayload) -> Result<String, ApiError> {  // ✅ Devuelve String
        self.schedule(payload, Utc::now(), 3).await
    }

    /// Schedule a job to be executed at a specific time
    pub async fn schedule(
        &self,
        payload: JobPayload,
        scheduled_at: DateTime<Utc>,
        max_attempts: i32,
    ) -> Result<String, ApiError> {  // ✅ Devuelve String
        let job_type = match &payload {
            JobPayload::DeleteTestItem { .. } => "DeleteTestItem",
            JobPayload::SendEmail { .. } => "SendEmail",
            JobPayload::ProcessPayment { .. } => "ProcessPayment",
            JobPayload::GenerateReport { .. } => "GenerateReport",
        };

        let payload_json = serde_json::to_value(&payload)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to serialize payload: {}", e)))?;

        // ✅ Generar UUID como String
        let job_id = Uuid::new_v4().to_string();

        let job: Job = sqlx::query_as(
            r#"
            INSERT INTO job_queue (id, job_type, payload, scheduled_at, max_attempts)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#
        )
        .bind(&job_id)  // ✅ Bind String ID
        .bind(job_type)
        .bind(payload_json)
        .bind(scheduled_at)
        .bind(max_attempts)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        tracing::info!("Job {} enqueued: {}", job.id, job_type);
        Ok(job.id)
    }

    /// Schedule a job to run after X seconds
    pub async fn enqueue_in(&self, payload: JobPayload, delay_seconds: i64) -> Result<String, ApiError> {  // ✅ String
        let scheduled_at = Utc::now() + Duration::seconds(delay_seconds);
        self.schedule(payload, scheduled_at, 3).await
    }

    /// Get pending jobs ready to be executed
    pub async fn get_pending_jobs(&self, limit: i64) -> Result<Vec<Job>, ApiError> {
        let jobs = sqlx::query_as::<_, Job>(
            r#"
            SELECT * FROM job_queue
            WHERE status = 'pending'
              AND scheduled_at <= NOW()
            ORDER BY priority DESC, scheduled_at ASC
            LIMIT $1
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(jobs)
    }

    /// Mark job as running
    pub async fn mark_running(&self, job_id: &str) -> Result<(), ApiError> {  // ✅ &str
        sqlx::query(
            r#"
            UPDATE job_queue
            SET status = 'running',
                started_at = NOW(),
                updated_at = NOW()
            WHERE id = $1
            "#
        )
        .bind(job_id)
        .execute(&self.pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Mark job as completed
    pub async fn mark_completed(&self, job_id: &str) -> Result<(), ApiError> {  // ✅ &str
        sqlx::query(
            r#"
            UPDATE job_queue
            SET status = 'completed',
                completed_at = NOW(),
                updated_at = NOW()
            WHERE id = $1
            "#
        )
        .bind(job_id)
        .execute(&self.pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Mark job as failed and retry if possible
    pub async fn mark_failed(&self, job_id: &str, error: &str) -> Result<(), ApiError> {  // ✅ &str
        let job: Job = sqlx::query_as(
            r#"
            UPDATE job_queue
            SET attempts = attempts + 1,
                error_message = $2,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(job_id)
        .bind(error)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        // If max attempts reached, mark as failed permanently
        if job.attempts >= job.max_attempts {
            sqlx::query(
                r#"
                UPDATE job_queue
                SET status = 'failed',
                    updated_at = NOW()
                WHERE id = $1
                "#
            )
            .bind(job_id)
            .execute(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

            tracing::error!("Job {} failed permanently after {} attempts", job_id, job.attempts);
        } else {
            // Reset to pending for retry
            sqlx::query(
                r#"
                UPDATE job_queue
                SET status = 'pending',
                    started_at = NULL,
                    updated_at = NOW()
                WHERE id = $1
                "#
            )
            .bind(job_id)
            .execute(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

            tracing::warn!("Job {} failed (attempt {}/{}), will retry", job_id, job.attempts, job.max_attempts);
        }

        Ok(())
    }

    /// Get queue statistics
    pub async fn get_statistics(&self) -> Result<serde_json::Value, ApiError> {
        let stats = sqlx::query(
            r#"
            SELECT 
                status,
                COUNT(*) as count
            FROM job_queue
            GROUP BY status
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        let mut result = serde_json::json!({
            "pending": 0,
            "running": 0,
            "completed": 0,
            "failed": 0,
            "cancelled": 0,
            "total": 0
        });

        let mut total = 0i64;

        for row in stats {
            let status: String = row.try_get("status")
                .map_err(|e| ApiError::DatabaseError(e.to_string()))?;
            let count: i64 = row.try_get("count")
                .map_err(|e| ApiError::DatabaseError(e.to_string()))?;
            
            result[status.to_lowercase()] = serde_json::json!(count);
            total += count;
        }

        result["total"] = serde_json::json!(total);

        Ok(result)
    }

    /// Retry a failed job
    pub async fn retry_job(&self, job_id: &str) -> Result<(), ApiError> {  // ✅ &str
        let result = sqlx::query(
            r#"
            UPDATE job_queue
            SET status = 'pending',
                attempts = 0,
                error_message = NULL,
                started_at = NULL,
                completed_at = NULL,
                updated_at = NOW()
            WHERE id = $1
              AND status IN ('failed', 'cancelled')
            "#
        )
        .bind(job_id)
        .execute(&self.pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(ApiError::NotFound(format!("Job {} not found or cannot be retried", job_id)));
        }

        tracing::info!("Job {} queued for retry", job_id);
        Ok(())
    }

    /// Cancel a pending job
    pub async fn cancel_job(&self, job_id: &str) -> Result<(), ApiError> {  // ✅ &str
        let result = sqlx::query(
            r#"
            UPDATE job_queue
            SET status = 'cancelled',
                updated_at = NOW()
            WHERE id = $1
              AND status = 'pending'
            "#
        )
        .bind(job_id)
        .execute(&self.pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(ApiError::NotFound(format!("Job {} not found or is not pending", job_id)));
        }

        tracing::info!("Job {} cancelled", job_id);
        Ok(())
    }
}