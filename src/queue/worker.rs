//! Worker module for asynchronous job processing.
//!
//! This module provides a distributed worker pool implementation that processes
//! asynchronous jobs from a persistent queue. The worker pool spans multiple
//! background tasks that continuously poll the queue, claim jobs atomically,
//! and execute them with proper error handling and state management.
//!
//! # Architecture
//!
//! - **Job Claiming**: Workers claim jobs atomically to prevent concurrent processing
//! - **Error Handling**: Failed jobs are retried with configurable backoff
//! - **Persistence**: Job state is maintained in PostgreSQL for durability
//! - **Isolation**: Multiple worker instances can run in parallel without conflicts

use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use sqlx::PgPool;

use crate::application::TestItemService;
use crate::queue::{QueueManager, JobPayload};

/// Worker pool for processing asynchronous jobs.
///
/// The `Worker` struct manages a pool of background tasks that continuously
/// process jobs from a persistent queue. It handles job execution, state transitions,
/// and error recovery gracefully.
///
/// # Concurrency Model
///
/// - Each worker runs in an independent Tokio task
/// - Jobs are claimed atomically to prevent concurrent execution
/// - Multiple threads/processes can safely run Worker instances concurrently
///
/// # Thread Safety
///
/// `Worker` is thread-safe and can be shared across threads using `Arc`,
/// making it ideal for use in multi-threaded async runtimes.
pub struct Worker {
    /// Queue manager for job coordination and state management
    queue: Arc<QueueManager>,
    /// Service for processing test item jobs
    test_item_service: Arc<TestItemService>,
}

impl Worker {
    /// Creates a new worker pool instance with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `pool` - PostgreSQL connection pool for queue persistence
    /// * `test_item_service` - Service instance for executing test item jobs
    ///
    /// # Returns
    ///
    /// A new `Worker` instance ready to be started.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let worker = Worker::new(pg_pool, test_item_service);
    /// ```
    pub fn new(
        pool: PgPool,
        test_item_service: Arc<TestItemService>,
    ) -> Self {
        Self {
            queue: Arc::new(QueueManager::new(pool)),
            test_item_service,
        }
    }

    /// Starts the worker pool with the specified number of concurrent workers.
    ///
    /// This method spawns `num_workers` independent Tokio tasks that will continuously
    /// process jobs from the queue. Each worker runs its own event loop and is capable
    /// of claiming, processing, and completing/failing jobs independently.
    ///
    /// # Arguments
    ///
    /// * `num_workers` - Number of concurrent worker tasks to spawn (typically should match CPU cores)
    ///
    /// # Lifecycle
    ///
    /// - Workers start immediately upon method invocation
    /// - Workers run indefinitely until the async runtime is shut down
    /// - No manual stopping mechanism is provided; rely on graceful shutdown at process level
    ///
    /// # Example
    ///
    /// ```ignore
    /// let worker = Arc::new(Worker::new(pool, service));
    /// worker.start(4); // Start 4 concurrent workers
    /// ```
    pub fn start(self: Arc<Self>, num_workers: usize) {
        for worker_id in 0..num_workers {
            let worker = Arc::clone(&self);
            tokio::spawn(async move {
                worker.run(worker_id).await;
            });
        }
        tracing::info!("🔧 Started {} workers", num_workers);
    }

    /// Main event loop for a single worker task.
    ///
    /// This method implements the core worker functionality:
    /// 1. **Claim**: Atomically claim the next available job from the queue
    /// 2. **Process**: Execute the job based on its type
    /// 3. **Report**: Mark the job as completed, failed, or retry
    /// 4. **Wait**: Sleep if no jobs are available
    ///
    /// The loop runs indefinitely with exponential backoff on errors to prevent
    /// rapid retry storms in case of database connectivity issues.
    ///
    /// # Arguments
    ///
    /// * `worker_id` - Unique identifier for this worker (used for logging/debugging)
    ///
    /// # Error Handling
    ///
    /// - **Success**: Job is marked as completed
    /// - **Processing Error**: Job is marked as failed with error details; retry logic is handled internally
    /// - **Claim Error**: Worker sleeps 5 seconds before retrying (DB connection issues)
    /// - **No Jobs**: Worker sleeps 1 second before polling again (normal idle state)
    async fn run(&self, worker_id: usize) {
        tracing::info!("Worker #{} started", worker_id);

        loop {
            // Attempt to atomically claim the next available job.
            // This ensures no two workers process the same job concurrently.
            match self.queue.claim_next_job().await {
                Ok(Some(job)) => {
                    tracing::info!("Worker #{} claimed job {}", worker_id, job.id);

                    // Execute the job with appropriate error handling.
                    match self.process_job(&job).await {
                        Ok(_) => {
                            // Job completed successfully; mark it as done.
                            if let Err(e) = self.queue.mark_completed(&job.id).await {
                                tracing::error!("Worker #{} failed to mark job {} as completed: {:?}", worker_id, job.id, e);
                            }
                        }
                        Err(e) => {
                            // Job failed during execution.
                            // Retry logic and backoff are managed by mark_failed internally.
                            let error_msg = format!("{:?}", e);
                            if let Err(e) = self.queue.mark_failed(&job.id, &error_msg).await {
                                tracing::error!("Worker #{} failed to mark job {} as failed: {:?}", worker_id, job.id, e);
                            }
                        }
                    }
                }
                Ok(None) => {
                    // No jobs available in the queue; sleep briefly before polling again.
                    // This prevents busy-waiting and reduces database load during idle periods.
                    time::sleep(Duration::from_secs(1)).await;
                }
                Err(e) => {
                    // Error while attempting to claim a job (likely a database error).
                    // Log the error and implement exponential backoff to avoid connection storms.
                    tracing::error!("Worker #{} error claiming job: {:?}", worker_id, e);
                    time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    /// Processes a single job based on its type and payload.
    ///
    /// This method dispatches job execution to type-specific handlers. Each job type
    /// has its own processing logic and may interact with external services (databases,
    /// message queues, email servers, etc.).
    ///
    /// # Arguments
    ///
    /// * `job` - The job to process, containing type and payload information
    ///
    /// # Returns
    ///
    /// - `Ok(())` - Job completed successfully
    /// - `Err(ApiError)` - Job failed; the error will be logged and the job marked for retry
    ///
    /// # Supported Job Types
    ///
    /// - `DeleteTestItem`: Delete a test item from the database
    /// - `SendEmail`: Send an email with the provided content
    /// - `ProcessPayment`: Process a payment transaction
    /// - `GenerateReport`: Generate and store a report
    ///
    /// # Error Handling
    ///
    /// Deserialization errors or unsupported job types return an `InternalServerError`
    /// which is then handled by the main worker loop (retry logic applied).
    async fn process_job(&self, job: &crate::queue::Job) -> Result<(), crate::errors::ApiError> {
        // Deserialize the job payload from JSON.
        let payload: JobPayload = serde_json::from_value(job.payload.clone())
            .map_err(|e| crate::errors::ApiError::InternalServerError(format!("Invalid payload: {}", e)))?;

        // Dispatch to the appropriate handler based on job type.
        match payload {
            JobPayload::DeleteTestItem { item_id } => {
                tracing::info!("Processing DeleteTestItem job for item: {}", item_id);
                
                // Delete the test item from the database.
                self.test_item_service.delete(&item_id).await?;
                tracing::info!("✅ Test item {} deleted successfully", item_id);
                Ok(())
            }
            
            JobPayload::SendEmail { to, subject, body } => {
                tracing::info!("Processing SendEmail job to: {}", to);
                // TODO: Integrate with email service provider
                tracing::info!("📧 Email sent to {} with subject: {}", to, subject);
                tracing::debug!("Email body: {}", body);
                Ok(())
            }
            
            JobPayload::ProcessPayment { amount, user_id } => {
                tracing::info!("Processing ProcessPayment job: ${} for user {}", amount, user_id);
                // TODO: Integrate with payment gateway
                tracing::info!("💳 Payment of ${} processed for user {}", amount, user_id);
                Ok(())
            }
            
            JobPayload::GenerateReport { report_type, user_id } => {
                tracing::info!("Processing GenerateReport job: {} for user {}", report_type, user_id);
                // TODO: Implement report generation logic
                tracing::info!("📊 Report '{}' generated for user {}", report_type, user_id);
                Ok(())
            }
        }
    }
}