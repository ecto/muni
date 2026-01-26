-- Phase 2: Alerts, Weather, Schedule, Zone polygon support

-- 1. Zone polygons for map display (lat/lng coordinates separate from local-frame waypoints)
ALTER TABLE zones ADD COLUMN polygon_latlng JSONB DEFAULT '[]';

-- 2. Alerts table for server-side persistent alerting
CREATE TABLE IF NOT EXISTS alerts (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    severity        TEXT NOT NULL,           -- critical, warning, info
    source          TEXT NOT NULL,           -- rover, service, dispatch, gps, weather
    source_id       TEXT NOT NULL,           -- rover ID, service name, etc.
    message         TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    acknowledged_at TIMESTAMPTZ,
    acknowledged_by TEXT,
    cleared_at      TIMESTAMPTZ,
    metadata        JSONB DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_alerts_active ON alerts (created_at DESC) WHERE cleared_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_alerts_source ON alerts (source, source_id);

-- 3. Schedule columns on missions (supplement JSONB schedule field)
ALTER TABLE missions ADD COLUMN schedule_type TEXT DEFAULT 'manual';
ALTER TABLE missions ADD COLUMN schedule_window_start TIME;
ALTER TABLE missions ADD COLUMN schedule_window_end TIME;

-- 4. Weather cache
CREATE TABLE IF NOT EXISTS weather_cache (
    id          SERIAL PRIMARY KEY,
    data        JSONB NOT NULL,
    fetched_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
