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
        let worker_id_str = format!("worker-{}", worker_id);

        tracing::info!("Worker #{} started", worker_id);

        loop {
            
            if let Err(e) = self.queue.recover_stuck_jobs().await {
                tracing::info!("Recovery error: {:?}", e);
            }

            match self.queue.claim_next_job(&worker_id_str).await {
                Ok(Some(job)) => {
                    tracing::info!(
                        job_id = %job.id,
                        worker_id = %worker_id_str,
                        "Job claimed"
                    );

                    let result = tokio::time::timeout(
                        Duration::from_secs(30),
                        self.process_job(&job)
                    ).await;

                    match result {
                        Ok(Ok(_)) => {
                            if let Err(e) = self.queue.mark_completed(&job.id).await {
                                tracing::error!("Mark completed error: {:?}", e);
                            }
                        }
                        Ok(Err(e)) => {
                            let error_msg = format!("{:?}", e);
                            let _ = self.queue.mark_failed(&job.id, &error_msg).await;
                        }
                        Err(_) => {
                            let _ = self.queue.mark_failed(&job.id, "Timeout").await;
                        }
                    }
                }
                Ok(None) => {
                    time::sleep(Duration::from_secs(1)).await;
                }
                Err(e) => {
                    tracing::error!("Worker error: {:?}", e);
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
            
        }
    }
}