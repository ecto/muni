# Database Patterns

Comprehensive guide for SQLx and PostgreSQL usage in depot services.

## Overview

Only the **dispatch** service uses PostgreSQL. Other services are stateless or use file-based storage.

**Technology Stack**:
- PostgreSQL 16+ (running in Docker)
- SQLx 0.8 (async PostgreSQL client with compile-time query checking)
- Migrations applied manually on startup

## Connection Pool Setup

### Creating the Pool

```rust
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

#[tokio::main]
async fn main() {
    // Load database URL from environment
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    // Create connection pool
    let pool = PgPoolOptions::new()
        .max_connections(10)  // Adjust based on expected load
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Run migrations
    run_migrations(&pool).await;

    // Store pool in app state
    let state = Arc::new(AppState { db: pool });
}
```

### Pool Configuration

**Connection Limits**:
- **Small services** (low traffic): 5-10 connections
- **Medium services** (moderate traffic): 10-20 connections
- **High traffic services**: 20-50 connections

**Why limit connections?**
- PostgreSQL has a default max of 100 connections
- Each connection consumes memory
- Too many connections can overload the database
- Pool prevents connection exhaustion

### Connection String Format

```bash
# Local development
DATABASE_URL=postgres://user:password@localhost:5432/database_name

# Docker Compose (service name as hostname)
DATABASE_URL=postgres://postgres:password@postgres:5432/dispatch

# Production (with SSL)
DATABASE_URL=postgres://user:password@host:5432/database?sslmode=require
```

## Migrations

### Migration File Structure

```
depot/dispatch/
├── src/
│   └── main.rs
└── migrations/
    ├── 001_initial.sql      # Initial schema
    ├── 002_add_index.sql    # Add indexes
    └── 003_alter_table.sql  # Schema changes
```

### Migration Naming Convention

```
{sequence}_{description}.sql

Examples:
001_initial.sql
002_add_mission_status.sql
003_create_logs_table.sql
```

### Migration Application

**Manual migrations** (current pattern in dispatch):

```rust
async fn run_migrations(pool: &PgPool) {
    // Embed migration at compile time
    let migration = include_str!("../migrations/001_initial.sql");

    // Check if migration already applied
    let table_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_name = 'zones'
        )"
    )
    .fetch_one(pool)
    .await
    .unwrap_or(false);

    if !table_exists {
        info!("Running initial migration...");
        sqlx::raw_sql(migration)
            .execute(pool)
            .await
            .expect("Failed to run migration");
        info!("Migration complete");
    } else {
        info!("Database already initialized");
    }
}
```

**Why manual migrations?**
- Simple for small services with few migrations
- No external migration tool needed
- Runs automatically on startup
- Easy to debug

**Limitations**:
- No rollback support
- No migration history tracking
- Requires manual idempotency checks
- Not suitable for many migrations

### Example Migration

```sql
-- migrations/001_initial.sql
-- Dispatch service initial schema

-- Zones: geographic areas for rover operations
CREATE TABLE zones (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL,
    zone_type   TEXT NOT NULL DEFAULT 'route',
    waypoints   JSONB NOT NULL,
    polygon     JSONB,
    map_id      UUID,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Missions: scheduled work definitions
CREATE TABLE missions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL,
    zone_id     UUID NOT NULL REFERENCES zones(id) ON DELETE CASCADE,
    rover_id    TEXT,
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
    status      TEXT NOT NULL DEFAULT 'pending',
    progress    INTEGER NOT NULL DEFAULT 0,
    waypoint    INTEGER NOT NULL DEFAULT 0,
    lap         INTEGER NOT NULL DEFAULT 0,
    error       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    started_at  TIMESTAMPTZ,
    ended_at    TIMESTAMPTZ
);

-- Indexes for common queries
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_rover ON tasks(rover_id);
CREATE INDEX idx_tasks_mission ON tasks(mission_id);
CREATE INDEX idx_missions_zone ON missions(zone_id);
```

## Query Patterns

### Basic Queries

**Select one row**:
```rust
let zone: Zone = sqlx::query_as(
    "SELECT id, name, zone_type, waypoints, polygon, map_id, created_at, updated_at
     FROM zones
     WHERE id = $1"
)
.bind(zone_id)
.fetch_one(&state.db)
.await
.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
```

**Select optional row**:
```rust
let zone: Option<Zone> = sqlx::query_as(
    "SELECT id, name, zone_type, waypoints, polygon, map_id, created_at, updated_at
     FROM zones
     WHERE id = $1"
)
.bind(zone_id)
.fetch_optional(&state.db)
.await
.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

// Convert to Result with custom error
let zone = zone.ok_or((StatusCode::NOT_FOUND, "Zone not found".to_string()))?;
```

**Select multiple rows**:
```rust
let zones: Vec<Zone> = sqlx::query_as(
    "SELECT id, name, zone_type, waypoints, polygon, map_id, created_at, updated_at
     FROM zones
     ORDER BY created_at DESC"
)
.fetch_all(&state.db)
.await
.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
```

### Insert Queries

**Insert with RETURNING**:
```rust
let zone: Zone = sqlx::query_as(
    r#"
    INSERT INTO zones (name, zone_type, waypoints, polygon, map_id)
    VALUES ($1, $2, $3, $4, $5)
    RETURNING id, name, zone_type, waypoints, polygon, map_id, created_at, updated_at
    "#
)
.bind(&payload.name)
.bind(&payload.zone_type)
.bind(&waypoints_json)
.bind(&polygon_json)
.bind(&payload.map_id)
.fetch_one(&state.db)
.await
.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
```

**Why RETURNING?**
- Returns the inserted row with generated fields (id, timestamps)
- Avoids separate SELECT query
- More efficient (single round-trip)

### Update Queries

**Update with RETURNING**:
```rust
let zone: Zone = sqlx::query_as(
    r#"
    UPDATE zones
    SET name = $2,
        zone_type = $3,
        waypoints = $4,
        polygon = $5,
        map_id = $6,
        updated_at = now()
    WHERE id = $1
    RETURNING id, name, zone_type, waypoints, polygon, map_id, created_at, updated_at
    "#
)
.bind(id)
.bind(&name)
.bind(&zone_type)
.bind(&waypoints_json)
.bind(&polygon_json)
.bind(&map_id)
.fetch_one(&state.db)
.await
.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
```

**Partial updates**:
```rust
// Fetch existing record first
let existing: Zone = sqlx::query_as("SELECT ... WHERE id = $1")
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or((StatusCode::NOT_FOUND, "Zone not found".to_string()))?;

// Merge with updates (use payload value if provided, else keep existing)
let name = payload.name.unwrap_or(existing.name);
let zone_type = payload.zone_type.unwrap_or(existing.zone_type);
let waypoints_json = if let Some(wps) = payload.waypoints {
    serde_json::to_value(&wps)?
} else {
    existing.waypoints
};

// Update with merged values
let zone: Zone = sqlx::query_as("UPDATE zones SET ... WHERE id = $1 RETURNING ...")
    .bind(id)
    .bind(&name)
    .bind(&zone_type)
    .bind(&waypoints_json)
    .fetch_one(&state.db)
    .await?;
```

### Delete Queries

```rust
let result = sqlx::query("DELETE FROM zones WHERE id = $1")
    .bind(id)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

if result.rows_affected() == 0 {
    return Err((StatusCode::NOT_FOUND, "Zone not found".to_string()));
}

Ok(StatusCode::NO_CONTENT)
```

### Complex Queries

**Filtering**:
```rust
let tasks: Vec<Task> = if let Some(status) = query.status {
    sqlx::query_as(
        "SELECT ... FROM tasks WHERE status = $1 ORDER BY created_at DESC"
    )
    .bind(status)
    .fetch_all(&state.db)
    .await
} else if let Some(rover_id) = query.rover_id {
    sqlx::query_as(
        "SELECT ... FROM tasks WHERE rover_id = $1 ORDER BY created_at DESC"
    )
    .bind(rover_id)
    .fetch_all(&state.db)
    .await
} else {
    sqlx::query_as(
        "SELECT ... FROM tasks ORDER BY created_at DESC LIMIT 100"
    )
    .fetch_all(&state.db)
    .await
}
.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
```

**Joins** (when needed):
```rust
// Example: Get mission with zone details
let result: (Mission, Zone) = sqlx::query_as(
    r#"
    SELECT
        m.id, m.name, m.zone_id, m.rover_id, m.schedule, m.enabled, m.created_at, m.updated_at,
        z.id, z.name, z.zone_type, z.waypoints, z.polygon, z.map_id, z.created_at, z.updated_at
    FROM missions m
    JOIN zones z ON m.zone_id = z.id
    WHERE m.id = $1
    "#
)
.bind(mission_id)
.fetch_one(&state.db)
.await?;
```

## Working with JSONB

### Database Model

```rust
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
struct Zone {
    pub id: Uuid,
    pub name: String,
    pub zone_type: String,
    #[sqlx(json)]  // SQLx will handle JSON serialization
    pub waypoints: serde_json::Value,  // Stored as JSONB
    #[sqlx(json)]
    pub polygon: Option<serde_json::Value>,
    pub map_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Inserting JSONB

```rust
// Rust struct
#[derive(Serialize, Deserialize)]
struct Waypoint {
    x: f64,
    y: f64,
    theta: Option<f64>,
}

// Convert to JSON value for database
let waypoints: Vec<Waypoint> = payload.waypoints;
let waypoints_json = serde_json::to_value(&waypoints)
    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

// Insert
sqlx::query("INSERT INTO zones (waypoints) VALUES ($1)")
    .bind(&waypoints_json)  // SQLx handles JSONB conversion
    .execute(&pool)
    .await?;
```

### Reading JSONB

```rust
// Query returns serde_json::Value
let zone: Zone = sqlx::query_as("SELECT ... FROM zones WHERE id = $1")
    .bind(id)
    .fetch_one(&pool)
    .await?;

// Parse JSON value into concrete type
let waypoints: Vec<Waypoint> = serde_json::from_value(zone.waypoints)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Invalid waypoints: {}", e)))?;

// Use waypoints
for wp in waypoints {
    println!("Waypoint at ({}, {})", wp.x, wp.y);
}
```

### Querying JSONB Fields

**Extract field**:
```sql
SELECT
    id,
    name,
    waypoints->'0'->>'x' as first_waypoint_x
FROM zones;
```

**Filter by JSONB field**:
```sql
SELECT * FROM zones
WHERE zone_type = 'route'
  AND waypoints @> '[{"x": 10.0}]'::jsonb;
```

**Check field exists**:
```sql
SELECT * FROM missions
WHERE schedule ? 'cron';  -- Has 'cron' key
```

## Transactions

**When to use transactions**:
- Multiple related INSERT/UPDATE/DELETE operations
- Need all-or-nothing semantics
- Prevent partial updates on failure

**When NOT to use transactions**:
- Single query (no benefit)
- Read-only queries (no data modification)
- Long-running operations (locks database)

### Basic Transaction

```rust
let mut tx = state.db.begin().await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

// Insert zone
let zone: Zone = sqlx::query_as(
    "INSERT INTO zones (...) VALUES (...) RETURNING ..."
)
.bind(...)
.fetch_one(&mut *tx)  // Use transaction
.await?;

// Insert mission referencing zone
let mission: Mission = sqlx::query_as(
    "INSERT INTO missions (zone_id, ...) VALUES ($1, ...) RETURNING ..."
)
.bind(zone.id)
.bind(...)
.fetch_one(&mut *tx)
.await?;

// Commit transaction
tx.commit().await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

Ok(Json((zone, mission)))
```

### Transaction Rollback

```rust
let mut tx = state.db.begin().await?;

let result = async {
    // Operations here
    sqlx::query("INSERT INTO zones ...")
        .execute(&mut *tx)
        .await?;

    // Something goes wrong
    if error_condition {
        return Err("Operation failed");
    }

    sqlx::query("INSERT INTO missions ...")
        .execute(&mut *tx)
        .await?;

    Ok(())
}.await;

match result {
    Ok(_) => {
        tx.commit().await?;
        Ok(StatusCode::CREATED)
    }
    Err(e) => {
        tx.rollback().await?;  // Explicit rollback
        Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
    }
}
```

**Automatic rollback**: If transaction is dropped without `commit()`, it automatically rolls back.

## Error Handling

### SQLx Error Types

```rust
use sqlx::Error as SqlxError;

match sqlx::query("...").execute(&pool).await {
    Ok(result) => { /* success */ },
    Err(SqlxError::RowNotFound) => {
        // No rows returned when expecting at least one
        return Err((StatusCode::NOT_FOUND, "Not found".to_string()));
    },
    Err(SqlxError::Database(e)) => {
        // Database error (constraint violation, syntax error, etc.)
        error!(error = %e, "Database error");
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()));
    },
    Err(SqlxError::PoolTimedOut) => {
        // Connection pool exhausted
        error!("Database pool timed out");
        return Err((StatusCode::SERVICE_UNAVAILABLE, "Service overloaded".to_string()));
    },
    Err(e) => {
        // Other errors (network, protocol, etc.)
        error!(error = %e, "Unexpected database error");
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()));
    }
}
```

### Constraint Violations

```rust
// Foreign key violation
sqlx::query("INSERT INTO missions (zone_id) VALUES ($1)")
    .bind(invalid_zone_id)
    .execute(&pool)
    .await
    .map_err(|e| {
        if let SqlxError::Database(db_err) = &e {
            if db_err.code() == Some(Cow::Borrowed("23503")) {  // FK violation
                return (StatusCode::BAD_REQUEST, "Zone not found".to_string());
            }
        }
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?;
```

Common PostgreSQL error codes:
- `23503` - Foreign key violation
- `23505` - Unique constraint violation
- `23514` - Check constraint violation

## Performance Optimization

### Indexes

**When to add indexes**:
- Columns frequently used in `WHERE` clauses
- Columns used in `JOIN` conditions
- Columns used in `ORDER BY`
- Foreign key columns

**Example**:
```sql
-- Good indexes for tasks table
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_rover ON tasks(rover_id);
CREATE INDEX idx_tasks_mission ON tasks(mission_id);

-- Composite index for common query
CREATE INDEX idx_tasks_rover_status ON tasks(rover_id, status);
```

### Query Optimization

**Use LIMIT for large result sets**:
```rust
let tasks: Vec<Task> = sqlx::query_as(
    "SELECT ... FROM tasks ORDER BY created_at DESC LIMIT 100"
)
.fetch_all(&pool)
.await?;
```

**Avoid N+1 queries** (use JOINs instead):
```rust
// ❌ BAD: N+1 queries
let missions: Vec<Mission> = sqlx::query_as("SELECT * FROM missions")
    .fetch_all(&pool)
    .await?;

for mission in missions {
    let zone: Zone = sqlx::query_as("SELECT * FROM zones WHERE id = $1")
        .bind(mission.zone_id)
        .fetch_one(&pool)
        .await?;
    // Use zone...
}

// ✅ GOOD: Single query with JOIN
let results: Vec<(Mission, Zone)> = sqlx::query_as(
    "SELECT m.*, z.* FROM missions m JOIN zones z ON m.zone_id = z.id"
)
.fetch_all(&pool)
.await?;
```

### Connection Pool Tuning

```rust
let pool = PgPoolOptions::new()
    .max_connections(20)           // Maximum connections
    .min_connections(5)            // Minimum connections to keep open
    .acquire_timeout(Duration::from_secs(3))  // Timeout waiting for connection
    .idle_timeout(Duration::from_secs(600))   // Close idle connections after 10 min
    .max_lifetime(Duration::from_secs(1800))  // Close connections after 30 min
    .connect(&database_url)
    .await?;
```

## Common Pitfalls

### 1. Not using parameterized queries

❌ **NEVER DO THIS** (SQL injection vulnerability):
```rust
let query = format!("SELECT * FROM zones WHERE name = '{}'", user_input);
sqlx::query(&query).fetch_all(&pool).await?;
```

✅ **ALWAYS DO THIS**:
```rust
sqlx::query_as("SELECT * FROM zones WHERE name = $1")
    .bind(user_input)
    .fetch_all(&pool)
    .await?;
```

### 2. Holding locks too long

❌ BAD:
```rust
let mut rovers = state.rovers.write().await;
// Heavy operation while holding lock
let result = expensive_database_query().await;
rovers.insert(id, result);
drop(rovers);
```

✅ GOOD:
```rust
let result = expensive_database_query().await;
let mut rovers = state.rovers.write().await;
rovers.insert(id, result);
drop(rovers);
```

### 3. Not handling NULL values

```rust
// Database column: rover_id TEXT NULL
#[derive(FromRow)]
struct Mission {
    pub rover_id: Option<String>,  // Must be Option for nullable columns
}
```

### 4. Forgetting to drop transaction

❌ BAD (holds transaction open):
```rust
let tx = state.db.begin().await?;
// Do something...
// Forgot to commit or rollback!
// Transaction stays open until timeout
```

✅ GOOD:
```rust
let mut tx = state.db.begin().await?;
// Do something...
tx.commit().await?;  // Or tx.rollback()
```

## Testing

### In-Memory Database for Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_db() -> PgPool {
        let pool = PgPoolOptions::new()
            .connect("postgres://postgres:password@localhost/test_db")
            .await
            .expect("Failed to connect to test database");

        // Run migrations
        sqlx::raw_sql(include_str!("../migrations/001_initial.sql"))
            .execute(&pool)
            .await
            .expect("Failed to run migrations");

        pool
    }

    #[tokio::test]
    async fn test_create_zone() {
        let pool = setup_test_db().await;

        let zone: Zone = sqlx::query_as(
            "INSERT INTO zones (name, zone_type, waypoints) VALUES ($1, $2, $3) RETURNING *"
        )
        .bind("Test Zone")
        .bind("route")
        .bind(serde_json::json!([{"x": 0.0, "y": 0.0}]))
        .fetch_one(&pool)
        .await
        .expect("Failed to create zone");

        assert_eq!(zone.name, "Test Zone");
    }
}
```
