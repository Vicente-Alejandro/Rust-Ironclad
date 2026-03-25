use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use serde_json;
use rand::Rng;

use crate::errors::ApiError;
use super::job::{Job, JobPayload};

use crate::monitoring::alerts::{Alert, AlertLevel};

// Alert service in /services
use crate::application::services::AlertService;

use std::env;

pub struct QueueManager {
    pool: PgPool,
}

impl QueueManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Enqueue a job to be executed immediately
    pub async fn enqueue(&self, payload: JobPayload) -> Result<String, ApiError> {
        self.schedule(payload, Utc::now(), 3).await
    }

    /// Schedule a job to be executed at a specific time
    pub async fn schedule(
        &self,
        payload: JobPayload,
        scheduled_at: DateTime<Utc>,
        max_attempts: i32,
    ) -> Result<String, ApiError> {
        let job_type = match &payload {
            JobPayload::DeleteTestItem { .. } => "DeleteTestItem",
        };

        let payload_json = serde_json::to_value(&payload)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to serialize payload: {}", e)))?;

        let job_id = Uuid::new_v4().to_string();

        let job: Job = sqlx::query_as(
            r#"
            INSERT INTO job_queue (id, job_type, payload, scheduled_at, max_attempts)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#
        )
        .bind(&job_id)
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
    pub async fn enqueue_in(&self, payload: JobPayload, delay_seconds: i64) -> Result<String, ApiError> {
        let scheduled_at = Utc::now() + Duration::seconds(delay_seconds);
        self.schedule(payload, scheduled_at, 3).await
    }

    /// Claim multiple jobs atomically (Batch Processing)
    pub async fn claim_next_jobs(
        &self,
        worker_id: &str,
        limit: i64
    ) -> Result<Vec<Job>, ApiError> {

        let jobs = sqlx::query_as::<_, Job>(
            r#"
            UPDATE job_queue
            SET status = 'running',
                started_at = NOW(),
                lock_expires_at = NOW() + interval '30 seconds',
                worker_id = $1,
                updated_at = NOW()
            WHERE id IN (
                SELECT id FROM job_queue
                WHERE status = 'pending'
                AND scheduled_at <= NOW()
                AND (retry_at IS NULL OR retry_at <= NOW())
                ORDER BY priority DESC, scheduled_at ASC
                LIMIT $2
                FOR UPDATE SKIP LOCKED
            )
            RETURNING *
            "#
        )
        .bind(worker_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(jobs)
    }

    /// Mark job as completed
    pub async fn mark_completed(&self, job_id: &str) -> Result<(), ApiError> {
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
    pub async fn mark_failed(&self, job_id: &str, error: &str) -> Result<(), ApiError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

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
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        if job.attempts >= job.max_attempts {

            // INSERT INTO DLQ
            sqlx::query(
                r#"
                INSERT INTO dead_letter_queue (
                    id,
                    original_job_id,
                    job_type,
                    payload,
                    error_message,
                    attempts,
                    max_attempts
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#
            )
            .bind(Uuid::new_v4().to_string())
            .bind(&job.id)
            .bind(&job.job_type)
            .bind(&job.payload)
            .bind(&job.error_message)
            .bind(job.attempts)
            .bind(job.max_attempts)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

            // Delete from main queue to prevent further retries
            sqlx::query(
                r#"
                DELETE FROM job_queue
                WHERE id = $1
                "#
            )
            .bind(job_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

            tracing::error!(
                job_id = %job_id,
                "Job moved to Dead Letter Queue"
            );

        } else {
            let delay = calculate_backoff(job.attempts);
            let retry_at = Utc::now() + Duration::seconds(delay);

            sqlx::query(
                r#"
                UPDATE job_queue
                SET status = 'pending',
                    retry_at = $2,
                    started_at = NULL,
                    lock_expires_at = NULL,
                    worker_id = NULL,
                    updated_at = NOW()
                WHERE id = $1
                "#
            )
            .bind(job_id)
            .bind(retry_at)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

            tracing::warn!(
                job_id = %job_id,
                retry_in = delay,
                "Retry scheduled"
            );
        }

        tx.commit().await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn requeue_from_dlq(&self, dlq_id: &str) -> Result<String, ApiError> {

        let mut tx = self.pool.begin().await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        let row = sqlx::query(
            r#"
            SELECT original_job_id, job_type, payload
            FROM dead_letter_queue
            WHERE id = $1
            "#
        )
        .bind(dlq_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        let row = match row {
            Some(r) => r,
            None => return Err(ApiError::NotFound("DLQ job not found".into()))
        };

        let job_type: String = row.try_get("job_type")
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        let payload: serde_json::Value = row.try_get("payload")
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        let new_id = Uuid::new_v4().to_string();

        sqlx::query(
            r#"
            INSERT INTO job_queue (id, job_type, payload, scheduled_at, max_attempts)
            VALUES ($1, $2, $3, NOW(), 3)
            "#
        )
        .bind(&new_id)
        .bind(&job_type)
        .bind(&payload)
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        sqlx::query(
            r#"
            DELETE FROM dead_letter_queue
            WHERE id = $1
            "#
        )
        .bind(dlq_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        tx.commit().await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        tracing::info!("DLQ job {} requeued as {}", dlq_id, new_id);

        Ok(new_id)
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
    pub async fn retry_job(&self, job_id: &str) -> Result<(), ApiError> {
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
    pub async fn cancel_job(&self, job_id: &str) -> Result<(), ApiError> {
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

    pub async fn recover_stuck_jobs(&self) -> Result<(), ApiError> {
        let affected = sqlx::query(
            r#"
            UPDATE job_queue
            SET status = 'pending',
                started_at = NULL,
                lock_expires_at = NULL,
                worker_id = NULL,
                updated_at = NOW()
            WHERE status = 'running'
            AND lock_expires_at < NOW()
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        if affected.rows_affected() > 0 {
            tracing::warn!("Recovered {} stuck jobs", affected.rows_affected());
        }

        Ok(())
    }

}

fn calculate_backoff(attempts: i32) -> i64 {
    use rand::Rng;

    let base_delay = env::var("QUEUE_BACKOFF_BASE")
        .unwrap_or_else(|_| "2".to_string())
        .parse::<i64>()
        .unwrap_or(2);
    let max_delay = env::var("QUEUE_BACKOFF_MAX")
        .unwrap_or_else(|_| "300".to_string())
        .parse::<i64>()
        .unwrap_or(300);

    // Exponential growth
    let exp_delay = base_delay * 2_i64.pow(attempts as u32);

    // Max cap to prevent excessively long delays
    let capped = std::cmp::min(exp_delay, max_delay);

    // Full jitter (0 → capped)
    let mut rng = rand::thread_rng();
    let jitter = rng.gen_range(0..=capped);

    jitter
}