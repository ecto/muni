-- Add missing indexes and constraints for query performance and data integrity

-- Index on tasks.created_at for time-range queries (task history, cleanup)
CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks(created_at DESC);

-- Index on missions.enabled for filtering active missions
CREATE INDEX IF NOT EXISTS idx_missions_enabled ON missions(enabled) WHERE enabled = true;

-- CHECK constraint on task status values
ALTER TABLE tasks ADD CONSTRAINT chk_tasks_status
    CHECK (status IN ('pending', 'assigned', 'active', 'done', 'failed', 'cancelled'));

-- CHECK constraint on task progress range
ALTER TABLE tasks ADD CONSTRAINT chk_tasks_progress
    CHECK (progress >= 0 AND progress <= 100);

-- CHECK constraint on zone type values
ALTER TABLE zones ADD CONSTRAINT chk_zones_zone_type
    CHECK (zone_type IN ('route', 'polygon', 'point'));

-- CHECK constraint on alert severity values
ALTER TABLE alerts ADD CONSTRAINT chk_alerts_severity
    CHECK (severity IN ('critical', 'warning', 'info'));
