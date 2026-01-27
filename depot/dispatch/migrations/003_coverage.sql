-- Coverage path planning support
-- Stores generated boustrophedon sweep waypoints for area coverage zones

ALTER TABLE zones ADD COLUMN coverage_config JSONB;
ALTER TABLE zones ADD COLUMN coverage_waypoints JSONB;
