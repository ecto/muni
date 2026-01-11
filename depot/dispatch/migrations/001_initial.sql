-- Dispatch service schema
-- Zones, Missions, and Tasks for rover dispatch

-- Zones: geographic areas to work on
CREATE TABLE zones (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL,
    zone_type   TEXT NOT NULL DEFAULT 'route',  -- route, polygon, point
    waypoints   JSONB NOT NULL,                  -- [{x, y, theta?}, ...] in map coords
    polygon     JSONB,                           -- GPS polygon for outdoor (future)
    map_id      UUID,                            -- Reference to map for indoor localization
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Missions: scheduled work definitions
CREATE TABLE missions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL,
    zone_id     UUID NOT NULL REFERENCES zones(id) ON DELETE CASCADE,
    rover_id    TEXT,                            -- NULL = any rover
    schedule    JSONB NOT NULL DEFAULT '{"trigger": "manual", "loop": false}',
    enabled     BOOL NOT NULL DEFAULT true,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Tasks: execution instances
CREATE TABLE tasks (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mission_id  UUID NOT NULL REFERENCES missions(id) ON DELETE CASCADE,
    rover_id    TEXT NOT NULL,
    status      TEXT NOT NULL DEFAULT 'pending', -- pending, assigned, active, done, failed, cancelled
    progress    INTEGER NOT NULL DEFAULT 0,      -- 0-100
    waypoint    INTEGER NOT NULL DEFAULT 0,      -- Current waypoint index
    lap         INTEGER NOT NULL DEFAULT 0,      -- For looping missions
    error       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    started_at  TIMESTAMPTZ,
    ended_at    TIMESTAMPTZ
);

CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_rover ON tasks(rover_id);
CREATE INDEX idx_tasks_mission ON tasks(mission_id);
CREATE INDEX idx_missions_zone ON missions(zone_id);
