use actix_web::{web, HttpResponse};
use std::sync::Arc;

use crate::errors::ApiResult;
use crate::queue::QueueManager;

use crate::errors::ApiError;
use sqlx::PgPool;

pub struct QueueController;

impl QueueController {
    pub async fn jobs_info(pool: web::Data<PgPool>) -> ApiResult<HttpResponse> {
        use sqlx::Row;

        // 1. Counters by status
        let status_counts = sqlx::query(
            r#"
            SELECT status, COUNT(*) as count
            FROM job_queue
            GROUP BY status
            "#
        )
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        let mut pending = 0;
        let mut processing = 0;
        let mut completed = 0;
        let mut failed = 0;

        for row in status_counts {
            let status: String = row.get("status");
            let count: i64 = row.get("count");

            match status.as_str() {
                "pending" => pending = count,
                "processing" => processing = count,
                "completed" => completed = count,
                "failed" => failed = count,
                _ => {}
            }
        }

        // 2. Stuck jobs (expired lock)
        let stuck_jobs: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM job_queue
            WHERE status = 'processing'
            AND lock_expires_at < NOW()
            "#
        )
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(0);

        // 3. Active workers
        let active_workers: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(DISTINCT worker_id)
            FROM job_queue
            WHERE status = 'processing'
            AND worker_id IS NOT NULL
            "#
        )
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(0);

        // 4. Average execution time
        let avg_time: Option<f64> = sqlx::query_scalar(
            r#"
            SELECT AVG(EXTRACT(EPOCH FROM (completed_at - started_at)))
            FROM job_queue
            WHERE completed_at IS NOT NULL
            "#
        )
        .fetch_one(pool.get_ref())
        .await
        .ok();

        #[derive(serde::Serialize)]
        struct JobsInfo {
            pending: i64,
            processing: i64,
            completed: i64,
            failed: i64,
            stuck_jobs: i64,
            active_workers: i64,
            avg_execution_seconds: Option<f64>,
        }

        let response = JobsInfo {
            pending,
            processing,
            completed,
            failed,
            stuck_jobs,
            active_workers,
            avg_execution_seconds: avg_time,
        };

        Ok(HttpResponse::Ok().json(response))
    }

    /// Get job statistics
    pub async fn get_stats(
        queue: web::Data<Arc<QueueManager>>,
    ) -> ApiResult<HttpResponse> {
        let stats = queue.get_statistics().await?;
        Ok(HttpResponse::Ok().json(stats))
    }

    /// Retry failed job
    pub async fn retry_job(
        queue: web::Data<Arc<QueueManager>>,
        job_id: web::Path<String>,  
    ) -> ApiResult<HttpResponse> {
        queue.retry_job(&job_id.into_inner()).await?;
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "ok",
            "message": "Job queued for retry"
        })))
    }

    /// Cancel pending job
    pub async fn cancel_job(
        queue: web::Data<Arc<QueueManager>>,
        job_id: web::Path<String>,  
    ) -> ApiResult<HttpResponse> {
        queue.cancel_job(&job_id.into_inner()).await?;
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "ok",
            "message": "Job cancelled"
        })))
    }

    pub async fn requeue_dlq(
        queue: web::Data<QueueManager>,
        path: web::Path<String>,
    ) -> ApiResult<HttpResponse> {

        let dlq_id = path.into_inner();

        let new_job_id = queue.requeue_from_dlq(&dlq_id).await?;

        Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "Job requeued successfully",
            "new_job_id": new_job_id
        })))
    }
}