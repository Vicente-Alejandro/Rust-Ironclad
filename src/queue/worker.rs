use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use sqlx::PgPool;

use crate::application::TestItemService;
use crate::queue::{QueueManager, JobPayload};

pub struct Worker {
    queue: Arc<QueueManager>,
    test_item_service: Arc<TestItemService>,
    // 🔮 Agregar más servicios cuando los necesites:
    // email_service: Arc<EmailService>,
    // payment_service: Arc<PaymentService>,
}

impl Worker {
    pub fn new(
        pool: PgPool,
        test_item_service: Arc<TestItemService>,
    ) -> Self {
        Self {
            queue: Arc::new(QueueManager::new(pool)),
            test_item_service,
        }
    }

    /// Start the worker pool
    pub fn start(self: Arc<Self>, num_workers: usize) {
        for worker_id in 0..num_workers {
            let worker = Arc::clone(&self);
            tokio::spawn(async move {
                worker.run(worker_id).await;
            });
        }
        tracing::info!("🔧 Started {} workers", num_workers);
    }

    /// Main worker loop
    async fn run(&self, worker_id: usize) {
        tracing::info!("Worker #{} started", worker_id);

        loop {
            match self.queue.get_pending_jobs(1).await {
                Ok(jobs) if !jobs.is_empty() => {
                    for job in jobs {
                        tracing::info!("Worker #{} processing job {}", worker_id, job.id);

                        // Mark job as running
                        if let Err(e) = self.queue.mark_running(&job.id).await {
                            tracing::error!("Worker #{} failed to mark job {} as running: {:?}", worker_id, job.id, e);
                            continue;
                        }

                        // Process the job
                        match self.process_job(&job).await {
                            Ok(_) => {
                                // Mark as completed
                                if let Err(e) = self.queue.mark_completed(&job.id).await {
                                    tracing::error!("Worker #{} failed to mark job {} as completed: {:?}", worker_id, job.id, e);
                                }
                            }
                            Err(e) => {
                                // Mark as failed (with retry logic inside)
                                let error_msg = format!("{:?}", e);
                                if let Err(e) = self.queue.mark_failed(&job.id, &error_msg).await {
                                    tracing::error!("Worker #{} failed to mark job {} as failed: {:?}", worker_id, job.id, e);
                                }
                            }
                        }
                    }
                }
                Ok(_) => {
                    // No jobs available, sleep for a bit
                    time::sleep(Duration::from_secs(1)).await;
                }
                Err(e) => {
                    // Error fetching jobs, log and retry after delay
                    tracing::error!("Worker #{} error fetching jobs: {:?}", worker_id, e);
                    time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    /// Process individual job based on its type
    async fn process_job(&self, job: &crate::queue::Job) -> Result<(), crate::errors::ApiError> {
        let payload: JobPayload = serde_json::from_value(job.payload.clone())
            .map_err(|e| crate::errors::ApiError::InternalServerError(format!("Invalid payload: {}", e)))?;

        match payload {
            JobPayload::DeleteTestItem { item_id } => {
                tracing::info!("Processing DeleteTestItem job for item: {}", item_id);
                
                // Execute the deletion
                match self.test_item_service.delete(&item_id).await {
                    Ok(_) => {
                        tracing::info!("✅ Test item {} deleted successfully", item_id);
                        Ok(())
                    }
                    Err(e) => {
                        tracing::error!("❌ Failed to delete test item {}: {:?}", item_id, e);
                        Err(e)
                    }
                }
            }
            
            JobPayload::SendEmail { to, subject, body } => {
                tracing::info!("Processing SendEmail job to: {}", to);
                
                // TODO: Implement email service
                // For now, just simulate success
                tracing::info!("📧 Email sent to {} with subject: {}", to, subject);
                tracing::debug!("Email body: {}", body);
                
                Ok(())
            }
            
            JobPayload::ProcessPayment { amount, user_id } => {
                tracing::info!("Processing ProcessPayment job: ${} for user {}", amount, user_id);
                
                // TODO: Implement payment service
                // For now, just simulate success
                tracing::info!("💳 Payment of ${} processed for user {}", amount, user_id);
                
                Ok(())
            }
            
            JobPayload::GenerateReport { report_type, user_id } => {
                tracing::info!("Processing GenerateReport job: {} for user {}", report_type, user_id);
                
                // TODO: Implement report service
                // For now, just simulate success
                tracing::info!("📊 Report '{}' generated for user {}", report_type, user_id);
                
                Ok(())
            }
        }
    }
}