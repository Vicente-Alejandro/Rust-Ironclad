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
use rand::Rng;
use tokio::time;
use sqlx::PgPool;

use crate::application::TestItemService;
use crate::queue::{QueueManager, JobPayload};
use crate::monitoring::queue_monitor::QueueMonitor;

use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::Semaphore;

use std::env;

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
    scheduler_index: AtomicUsize,
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
            scheduler_index: AtomicUsize::new(0),
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
    async fn run(self: Arc<Self>, worker_id: usize) {
        let worker_id_str = format!("worker-{}", worker_id);
        let batch_size = env::var("QUEUE_BATCH_SIZE")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<i64>()
            .unwrap_or(5);

        let semaphore = Arc::new(Semaphore::new(11));
        let monitor = QueueMonitor::new(self.queue.pool().clone());
        let mut last_check = std::time::Instant::now();

        tracing::info!("Worker #{} started", worker_id);
        
        loop {
            // Recover stuck jobs
            if let Err(e) = self.queue.recover_stuck_jobs().await {
                tracing::warn!("Recovery error: {:?}", e);
            }
            // Check alerts every 10 seconds
            if worker_id == 0 && last_check.elapsed().as_secs() > 10 {
                if let Err(e) = monitor.check_alerts().await {
                    tracing::error!("Alert check error: {:?}", e);
                }
                last_check = std::time::Instant::now();
            }

            // Claim batch
            let weighted_queues = vec![
                ("critical", 5),
                ("default", 3),
                ("low", 2),
            ];

            let mut queue_cycle = Vec::new();
            for (queue, weight) in &weighted_queues {
                for _ in 0..*weight {
                    queue_cycle.push(*queue);
                }
            }

            let index = self.scheduler_index.fetch_add(1, Ordering::Relaxed);
            let queue = queue_cycle[index % queue_cycle.len()];

            let queues = vec![queue];

            // Claim con multi-queue
            match self.queue.claim_next_jobs(&worker_id_str, &queues, batch_size).await {
                Ok(jobs) if !jobs.is_empty() => {
                    for job in jobs {
                        let queue = self.queue.clone();
                        let worker = Arc::clone(&self);
                        let semaphore = semaphore.clone();

                        tokio::spawn(async move {
                            let permit = semaphore.acquire_owned().await.unwrap();

                            let result = tokio::time::timeout(
                                Duration::from_secs(30),
                                worker.process_job(&job)
                            ).await;

                            match result {
                                Ok(Ok(_)) => {
                                    let _ = queue.mark_completed(&job.id).await;
                                }
                                Ok(Err(e)) => {
                                    let _ = queue.mark_failed(&job.id, &format!("{:?}", e)).await;
                                }
                                Err(_) => {
                                    let _ = queue.mark_failed(&job.id, "Timeout").await;
                                }
                            }

                            drop(permit); // Free the permit after processing
                        });
                    }
                }
                Ok(_) => {
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