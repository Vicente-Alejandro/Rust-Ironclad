use actix_web::{web, HttpResponse};
use std::sync::Arc;

use crate::errors::ApiResult;
use crate::queue::QueueManager;

pub struct QueueController;

impl QueueController {
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
}