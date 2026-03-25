-- Drop table if exists 
DROP TABLE IF EXISTS job_queue;

-- Create job_queue table with VARCHAR ID (consistent with test_items/users)
CREATE TABLE job_queue (
    id VARCHAR(36) PRIMARY KEY,  
    job_type VARCHAR(100) NOT NULL,
    payload JSONB NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    priority INT NOT NULL DEFAULT 0,
    scheduled_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    attempts INT NOT NULL DEFAULT 0,
    max_attempts INT NOT NULL DEFAULT 3,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_job_queue_status ON job_queue(status);
CREATE INDEX idx_job_queue_scheduled_at ON job_queue(scheduled_at);
CREATE INDEX idx_job_queue_priority ON job_queue(priority DESC, scheduled_at ASC);

-- Function to update updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Trigger to auto-update updated_at
CREATE TRIGGER update_job_queue_updated_at BEFORE UPDATE ON job_queue
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

ALTER TABLE job_queue 
ADD COLUMN retry_at TIMESTAMPTZ,
ADD COLUMN lock_expires_at TIMESTAMPTZ,
ADD COLUMN worker_id VARCHAR(100);

CREATE INDEX idx_jobs_status_scheduled
ON jobs(status, scheduled_at);