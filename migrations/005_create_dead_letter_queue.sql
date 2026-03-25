CREATE TABLE dead_letter_queue (
    id VARCHAR(36) PRIMARY KEY,
    original_job_id VARCHAR(36) NOT NULL,
    job_type VARCHAR(100) NOT NULL,
    payload JSONB NOT NULL,
    error_message TEXT,
    failed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    attempts INT NOT NULL,
    max_attempts INT NOT NULL
);

CREATE INDEX idx_dlq_failed_at ON dead_letter_queue(failed_at);