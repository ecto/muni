//! Dispatch Service
//!
//! Mission planning and task assignment for rover fleet operations.
//!
//! Endpoints:
//! - Zone CRUD: POST/GET/PUT/DELETE /zones
//! - Mission CRUD: POST/GET/PUT/DELETE /missions
//! - Mission control: POST /missions/:id/start, POST /missions/:id/stop
//! - Task management: GET /tasks, POST /tasks/:id/cancel
//! - WebSocket: /ws - rovers connect here for task assignment
//! - Health: GET /health

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, FromRow, PgPool};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::{broadcast, RwLock};
use tower_http::cors::CorsLayer;
use tracing::{info, warn};
use uuid::Uuid;

// =============================================================================
// Database Models
// =============================================================================

/// Waypoint in a zone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Waypoint {
    pub x: f64,
    pub y: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theta: Option<f64>,
}

/// Zone record from database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Zone {
    pub id: Uuid,
    pub name: String,
    pub zone_type: String,
    #[sqlx(json)]
    pub waypoints: serde_json::Value,
    #[sqlx(json)]
    pub polygon: Option<serde_json::Value>,
    pub map_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// GPS coordinate for outdoor zones
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpsCoord {
    pub lat: f64,
    pub lon: f64,
}

/// Mission schedule configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Schedule {
    /// Trigger type: "manual", "once", "cron"
    #[serde(default = "default_trigger")]
    pub trigger: String,
    /// Cron expression (if trigger = "cron")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cron: Option<String>,
    /// Whether to loop continuously
    #[serde(default)]
    pub r#loop: bool,
}

fn default_trigger() -> String {
    "manual".to_string()
}

/// Mission record from database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Mission {
    pub id: Uuid,
    pub name: String,
    pub zone_id: Uuid,
    pub rover_id: Option<String>,
    #[sqlx(json)]
    pub schedule: serde_json::Value,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Assigned,
    Active,
    Done,
    Failed,
    Cancelled,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "pending"),
            TaskStatus::Assigned => write!(f, "assigned"),
            TaskStatus::Active => write!(f, "active"),
            TaskStatus::Done => write!(f, "done"),
            TaskStatus::Failed => write!(f, "failed"),
            TaskStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl From<String> for TaskStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "pending" => TaskStatus::Pending,
            "assigned" => TaskStatus::Assigned,
            "active" => TaskStatus::Active,
            "done" => TaskStatus::Done,
            "failed" => TaskStatus::Failed,
            "cancelled" => TaskStatus::Cancelled,
            _ => TaskStatus::Pending,
        }
    }
}



/// Task record from database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: Uuid,
    pub mission_id: Uuid,
    pub rover_id: String,
    pub status: String,
    pub progress: i32,
    pub waypoint: i32,
    pub lap: i32,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
}

// =============================================================================
// API Request/Response Types
// =============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateZone {
    pub name: String,
    #[serde(default = "default_zone_type")]
    pub zone_type: String,
    pub waypoints: Vec<Waypoint>,
    pub polygon: Option<Vec<GpsCoord>>,
    pub map_id: Option<Uuid>,
}

fn default_zone_type() -> String {
    "route".to_string()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateZone {
    pub name: Option<String>,
    pub zone_type: Option<String>,
    pub waypoints: Option<Vec<Waypoint>>,
    pub polygon: Option<Vec<GpsCoord>>,
    pub map_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMission {
    pub name: String,
    pub zone_id: Uuid,
    pub rover_id: Option<String>,
    #[serde(default)]
    pub schedule: Schedule,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMission {
    pub name: Option<String>,
    pub zone_id: Option<Uuid>,
    pub rover_id: Option<String>,
    pub schedule: Option<Schedule>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct TasksQuery {
    pub status: Option<String>,
    pub rover_id: Option<String>,
    pub mission_id: Option<Uuid>,
}

// =============================================================================
// WebSocket Protocol
// =============================================================================

/// Message from dispatch to rover
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum DispatchToRover {
    /// Assign a task to the rover
    Task {
        task_id: Uuid,
        mission_id: Uuid,
        zone: ZoneData,
    },
    /// Cancel the current task
    Cancel { task_id: Uuid },
}

/// Zone data sent to rover
#[derive(Debug, Clone, Serialize)]
pub struct ZoneData {
    pub waypoints: Vec<Waypoint>,
    pub r#loop: bool,
}

/// Message from rover to dispatch
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RoverToDispatch {
    /// Rover registration
    Register { rover_id: String },
    /// Task progress update
    Progress {
        task_id: Uuid,
        progress: i32,
        waypoint: i32,
        lap: i32,
    },
    /// Task completed
    Complete { task_id: Uuid, laps: i32 },
    /// Task failed
    Failed { task_id: Uuid, error: String },
}

/// Broadcast message to console clients
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BroadcastMessage {
    TaskUpdate { task: Task },
    RoverUpdate {
        rover_id: String,
        connected: bool,
        task_id: Option<Uuid>,
    },
    ZoneUpdate { zone: Zone },
    MissionUpdate { mission: Mission },
}

// =============================================================================
// Application State
// =============================================================================

/// Connected rover info
#[derive(Debug, Clone)]
pub struct ConnectedRover {
    pub rover_id: String,
    pub current_task: Option<Uuid>,
    pub tx: tokio::sync::mpsc::Sender<DispatchToRover>,
}

/// Shared application state
pub struct AppState {
    pub db: PgPool,
    /// Connected rovers (rover_id -> connection info)
    pub rovers: RwLock<HashMap<String, ConnectedRover>>,
    /// Broadcast channel for console updates
    pub broadcast_tx: broadcast::Sender<BroadcastMessage>,
}

impl AppState {
    pub fn new(db: PgPool) -> Self {
        let (broadcast_tx, _) = broadcast::channel(256);
        Self {
            db,
            rovers: RwLock::new(HashMap::new()),
            broadcast_tx,
        }
    }

    /// Broadcast an update to all console clients
    pub fn broadcast(&self, msg: BroadcastMessage) {
        let _ = self.broadcast_tx.send(msg);
    }

    /// Send a message to a specific rover
    pub async fn send_to_rover(&self, rover_id: &str, msg: DispatchToRover) -> bool {
        let rovers = self.rovers.read().await;
        if let Some(rover) = rovers.get(rover_id) {
            rover.tx.send(msg).await.is_ok()
        } else {
            false
        }
    }
}

type SharedState = Arc<AppState>;

// =============================================================================
// Main
// =============================================================================

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "dispatch=info,sqlx=warn".into()),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Run migrations
    info!("Running migrations...");
    run_migrations(&pool).await;

    let state = Arc::new(AppState::new(pool));

    let app = Router::new()
        // Zone endpoints
        .route("/zones", post(create_zone))
        .route("/zones", get(list_zones))
        .route("/zones/{id}", get(get_zone))
        .route("/zones/{id}", put(update_zone))
        .route("/zones/{id}", delete(delete_zone))
        // Mission endpoints
        .route("/missions", post(create_mission))
        .route("/missions", get(list_missions))
        .route("/missions/{id}", get(get_mission))
        .route("/missions/{id}", put(update_mission))
        .route("/missions/{id}", delete(delete_mission))
        .route("/missions/{id}/start", post(start_mission))
        .route("/missions/{id}/stop", post(stop_mission))
        // Task endpoints
        .route("/tasks", get(list_tasks))
        .route("/tasks/{id}", get(get_task))
        .route("/tasks/{id}/cancel", post(cancel_task))
        // WebSocket
        .route("/ws", get(ws_handler))
        .route("/ws/console", get(ws_console_handler))
        // Health
        .route("/health", get(health))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(4890);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Dispatch service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Run database migrations manually
async fn run_migrations(pool: &PgPool) {
    let migration = include_str!("../migrations/001_initial.sql");

    // Check if migrations have been run by checking for zones table
    let table_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'zones')",
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

// =============================================================================
// Health Check
// =============================================================================

async fn health(State(state): State<SharedState>) -> impl IntoResponse {
    let rovers = state.rovers.read().await;
    Json(serde_json::json!({
        "status": "ok",
        "connected_rovers": rovers.len()
    }))
}

// =============================================================================
// Zone Endpoints
// =============================================================================

async fn create_zone(
    State(state): State<SharedState>,
    Json(payload): Json<CreateZone>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let waypoints_json =
        serde_json::to_value(&payload.waypoints).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let polygon_json = payload
        .polygon
        .as_ref()
        .map(|p| serde_json::to_value(p))
        .transpose()
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let zone: Zone = sqlx::query_as(
        r#"
        INSERT INTO zones (name, zone_type, waypoints, polygon, map_id)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, name, zone_type, waypoints, polygon, map_id, created_at, updated_at
        "#,
    )
    .bind(&payload.name)
    .bind(&payload.zone_type)
    .bind(&waypoints_json)
    .bind(&polygon_json)
    .bind(&payload.map_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    info!(id = %zone.id, name = %zone.name, "Zone created");
    state.broadcast(BroadcastMessage::ZoneUpdate { zone: zone.clone() });

    Ok((StatusCode::CREATED, Json(zone)))
}

async fn list_zones(State(state): State<SharedState>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let zones: Vec<Zone> = sqlx::query_as(
        "SELECT id, name, zone_type, waypoints, polygon, map_id, created_at, updated_at FROM zones ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(zones))
}

async fn get_zone(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let zone: Zone = sqlx::query_as(
        "SELECT id, name, zone_type, waypoints, polygon, map_id, created_at, updated_at FROM zones WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Zone not found".to_string()))?;

    Ok(Json(zone))
}

async fn update_zone(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateZone>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // First fetch existing zone
    let existing: Zone = sqlx::query_as(
        "SELECT id, name, zone_type, waypoints, polygon, map_id, created_at, updated_at FROM zones WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Zone not found".to_string()))?;

    let name = payload.name.unwrap_or(existing.name);
    let zone_type = payload.zone_type.unwrap_or(existing.zone_type);
    let waypoints_json = if let Some(wps) = payload.waypoints {
        serde_json::to_value(&wps).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    } else {
        existing.waypoints
    };
    let polygon_json = if payload.polygon.is_some() {
        payload
            .polygon
            .as_ref()
            .map(|p| serde_json::to_value(p))
            .transpose()
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    } else {
        existing.polygon
    };
    let map_id = payload.map_id.or(existing.map_id);

    let zone: Zone = sqlx::query_as(
        r#"
        UPDATE zones 
        SET name = $2, zone_type = $3, waypoints = $4, polygon = $5, map_id = $6, updated_at = now()
        WHERE id = $1
        RETURNING id, name, zone_type, waypoints, polygon, map_id, created_at, updated_at
        "#,
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

    info!(id = %zone.id, "Zone updated");
    state.broadcast(BroadcastMessage::ZoneUpdate { zone: zone.clone() });

    Ok(Json(zone))
}

async fn delete_zone(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM zones WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Zone not found".to_string()));
    }

    info!(id = %id, "Zone deleted");
    Ok(StatusCode::NO_CONTENT)
}

// =============================================================================
// Mission Endpoints
// =============================================================================

async fn create_mission(
    State(state): State<SharedState>,
    Json(payload): Json<CreateMission>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Verify zone exists
    let zone_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM zones WHERE id = $1)")
            .bind(payload.zone_id)
            .fetch_one(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !zone_exists {
        return Err((StatusCode::BAD_REQUEST, "Zone not found".to_string()));
    }

    let schedule_json =
        serde_json::to_value(&payload.schedule).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let mission: Mission = sqlx::query_as(
        r#"
        INSERT INTO missions (name, zone_id, rover_id, schedule)
        VALUES ($1, $2, $3, $4)
        RETURNING id, name, zone_id, rover_id, schedule, enabled, created_at, updated_at
        "#,
    )
    .bind(&payload.name)
    .bind(&payload.zone_id)
    .bind(&payload.rover_id)
    .bind(&schedule_json)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    info!(id = %mission.id, name = %mission.name, "Mission created");
    state.broadcast(BroadcastMessage::MissionUpdate {
        mission: mission.clone(),
    });

    Ok((StatusCode::CREATED, Json(mission)))
}

async fn list_missions(
    State(state): State<SharedState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let missions: Vec<Mission> = sqlx::query_as(
        "SELECT id, name, zone_id, rover_id, schedule, enabled, created_at, updated_at FROM missions ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(missions))
}

async fn get_mission(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mission: Mission = sqlx::query_as(
        "SELECT id, name, zone_id, rover_id, schedule, enabled, created_at, updated_at FROM missions WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Mission not found".to_string()))?;

    Ok(Json(mission))
}

async fn update_mission(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateMission>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let existing: Mission = sqlx::query_as(
        "SELECT id, name, zone_id, rover_id, schedule, enabled, created_at, updated_at FROM missions WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Mission not found".to_string()))?;

    let name = payload.name.unwrap_or(existing.name);
    let zone_id = payload.zone_id.unwrap_or(existing.zone_id);
    let rover_id = if payload.rover_id.is_some() {
        payload.rover_id
    } else {
        existing.rover_id
    };
    let schedule_json = if let Some(sched) = payload.schedule {
        serde_json::to_value(&sched).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    } else {
        existing.schedule
    };
    let enabled = payload.enabled.unwrap_or(existing.enabled);

    let mission: Mission = sqlx::query_as(
        r#"
        UPDATE missions
        SET name = $2, zone_id = $3, rover_id = $4, schedule = $5, enabled = $6, updated_at = now()
        WHERE id = $1
        RETURNING id, name, zone_id, rover_id, schedule, enabled, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(&name)
    .bind(&zone_id)
    .bind(&rover_id)
    .bind(&schedule_json)
    .bind(enabled)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    info!(id = %mission.id, "Mission updated");
    state.broadcast(BroadcastMessage::MissionUpdate {
        mission: mission.clone(),
    });

    Ok(Json(mission))
}

async fn delete_mission(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM missions WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Mission not found".to_string()));
    }

    info!(id = %id, "Mission deleted");
    Ok(StatusCode::NO_CONTENT)
}

async fn start_mission(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Get mission with zone data
    let mission: Mission = sqlx::query_as(
        "SELECT id, name, zone_id, rover_id, schedule, enabled, created_at, updated_at FROM missions WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Mission not found".to_string()))?;

    let zone: Zone = sqlx::query_as(
        "SELECT id, name, zone_type, waypoints, polygon, map_id, created_at, updated_at FROM zones WHERE id = $1",
    )
    .bind(mission.zone_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Determine which rover to assign
    let rover_id = if let Some(ref preferred) = mission.rover_id {
        // Check if preferred rover is connected
        let rovers = state.rovers.read().await;
        if rovers.contains_key(preferred) {
            preferred.clone()
        } else {
            return Err((
                StatusCode::CONFLICT,
                format!("Rover {} not connected", preferred),
            ));
        }
    } else {
        // Find any available connected rover
        let rovers = state.rovers.read().await;
        let available = rovers
            .iter()
            .find(|(_, r)| r.current_task.is_none())
            .map(|(id, _)| id.clone());

        available.ok_or((StatusCode::CONFLICT, "No available rovers".to_string()))?
    };

    // Check if rover already has an active task
    {
        let rovers = state.rovers.read().await;
        if let Some(rover) = rovers.get(&rover_id) {
            if rover.current_task.is_some() {
                return Err((
                    StatusCode::CONFLICT,
                    "Rover already has an active task".to_string(),
                ));
            }
        }
    }

    // Create task
    let schedule: Schedule = serde_json::from_value(mission.schedule.clone()).unwrap_or_default();

    let task: Task = sqlx::query_as(
        r#"
        INSERT INTO tasks (mission_id, rover_id, status)
        VALUES ($1, $2, 'assigned')
        RETURNING id, mission_id, rover_id, status, progress, waypoint, lap, error, created_at, started_at, ended_at
        "#,
    )
    .bind(id)
    .bind(&rover_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    info!(task_id = %task.id, rover_id = %rover_id, mission_id = %id, "Task created and assigned");

    // Update rover's current task
    {
        let mut rovers = state.rovers.write().await;
        if let Some(rover) = rovers.get_mut(&rover_id) {
            rover.current_task = Some(task.id);
        }
    }

    // Parse waypoints from zone
    let waypoints: Vec<Waypoint> = serde_json::from_value(zone.waypoints.clone()).unwrap_or_default();

    // Send task to rover
    let msg = DispatchToRover::Task {
        task_id: task.id,
        mission_id: id,
        zone: ZoneData {
            waypoints,
            r#loop: schedule.r#loop,
        },
    };

    if !state.send_to_rover(&rover_id, msg).await {
        // Failed to send, rollback
        warn!(rover_id = %rover_id, "Failed to send task to rover");
        sqlx::query("DELETE FROM tasks WHERE id = $1")
            .bind(task.id)
            .execute(&state.db)
            .await
            .ok();
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to send task to rover".to_string(),
        ));
    }

    // Broadcast update
    state.broadcast(BroadcastMessage::TaskUpdate { task: task.clone() });
    state.broadcast(BroadcastMessage::RoverUpdate {
        rover_id: rover_id.clone(),
        connected: true,
        task_id: Some(task.id),
    });

    Ok((StatusCode::CREATED, Json(task)))
}

async fn stop_mission(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Find active task for this mission
    let task: Task = sqlx::query_as(
        r#"
        SELECT id, mission_id, rover_id, status, progress, waypoint, lap, error, created_at, started_at, ended_at
        FROM tasks 
        WHERE mission_id = $1 AND status IN ('assigned', 'active')
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((
        StatusCode::NOT_FOUND,
        "No active task for this mission".to_string(),
    ))?;

    // Cancel the task
    cancel_task_internal(&state, task.id).await
}

// =============================================================================
// Task Endpoints
// =============================================================================

async fn list_tasks(
    State(state): State<SharedState>,
    Query(query): Query<TasksQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let tasks: Vec<Task> = if let Some(status) = query.status {
        sqlx::query_as(
            r#"
            SELECT id, mission_id, rover_id, status, progress, waypoint, lap, error, created_at, started_at, ended_at
            FROM tasks WHERE status = $1 ORDER BY created_at DESC
            "#,
        )
        .bind(status)
        .fetch_all(&state.db)
        .await
    } else if let Some(rover_id) = query.rover_id {
        sqlx::query_as(
            r#"
            SELECT id, mission_id, rover_id, status, progress, waypoint, lap, error, created_at, started_at, ended_at
            FROM tasks WHERE rover_id = $1 ORDER BY created_at DESC
            "#,
        )
        .bind(rover_id)
        .fetch_all(&state.db)
        .await
    } else if let Some(mission_id) = query.mission_id {
        sqlx::query_as(
            r#"
            SELECT id, mission_id, rover_id, status, progress, waypoint, lap, error, created_at, started_at, ended_at
            FROM tasks WHERE mission_id = $1 ORDER BY created_at DESC
            "#,
        )
        .bind(mission_id)
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query_as(
            r#"
            SELECT id, mission_id, rover_id, status, progress, waypoint, lap, error, created_at, started_at, ended_at
            FROM tasks ORDER BY created_at DESC LIMIT 100
            "#,
        )
        .fetch_all(&state.db)
        .await
    }
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(tasks))
}

async fn get_task(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let task: Task = sqlx::query_as(
        r#"
        SELECT id, mission_id, rover_id, status, progress, waypoint, lap, error, created_at, started_at, ended_at
        FROM tasks WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Task not found".to_string()))?;

    Ok(Json(task))
}

async fn cancel_task(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    cancel_task_internal(&state, id).await
}

async fn cancel_task_internal(
    state: &AppState,
    task_id: Uuid,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let task: Task = sqlx::query_as(
        r#"
        UPDATE tasks 
        SET status = 'cancelled', ended_at = now()
        WHERE id = $1 AND status IN ('pending', 'assigned', 'active')
        RETURNING id, mission_id, rover_id, status, progress, waypoint, lap, error, created_at, started_at, ended_at
        "#,
    )
    .bind(task_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((
        StatusCode::NOT_FOUND,
        "Task not found or already completed".to_string(),
    ))?;

    let rover_id = task.rover_id.clone();

    info!(task_id = %task_id, rover_id = %rover_id, "Task cancelled");

    // Clear rover's current task
    {
        let mut rovers = state.rovers.write().await;
        if let Some(rover) = rovers.get_mut(&rover_id) {
            rover.current_task = None;
        }
    }

    // Send cancel to rover
    state
        .send_to_rover(&rover_id, DispatchToRover::Cancel { task_id })
        .await;

    // Broadcast update
    state.broadcast(BroadcastMessage::TaskUpdate { task: task.clone() });
    state.broadcast(BroadcastMessage::RoverUpdate {
        rover_id,
        connected: true,
        task_id: None,
    });

    Ok(Json(task))
}

// =============================================================================
// WebSocket Handlers
// =============================================================================

/// WebSocket handler for rovers
async fn ws_handler(ws: WebSocketUpgrade, State(state): State<SharedState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_rover_ws(socket, state))
}

async fn handle_rover_ws(socket: WebSocket, state: SharedState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<DispatchToRover>(32);

    let mut rover_id: Option<String> = None;

    // Spawn task to forward messages to rover
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        let msg = match msg {
            Ok(Message::Text(text)) => text,
            Ok(Message::Close(_)) => break,
            Ok(Message::Ping(_)) => {
                // Pong is handled automatically by axum
                continue;
            }
            _ => continue,
        };

        let parsed: Result<RoverToDispatch, _> = serde_json::from_str(&msg);
        match parsed {
            Ok(RoverToDispatch::Register { rover_id: id }) => {
                info!(rover_id = %id, "Rover connected");
                rover_id = Some(id.clone());

                // Register rover
                let mut rovers = state.rovers.write().await;
                rovers.insert(
                    id.clone(),
                    ConnectedRover {
                        rover_id: id.clone(),
                        current_task: None,
                        tx: tx.clone(),
                    },
                );
                drop(rovers);

                // Broadcast connection
                state.broadcast(BroadcastMessage::RoverUpdate {
                    rover_id: id,
                    connected: true,
                    task_id: None,
                });
            }
            Ok(RoverToDispatch::Progress {
                task_id,
                progress,
                waypoint,
                lap,
            }) => {
                // Update task in database
                let result: Result<Task, _> = sqlx::query_as(
                    r#"
                    UPDATE tasks 
                    SET progress = $2, waypoint = $3, lap = $4, status = 'active', started_at = COALESCE(started_at, now())
                    WHERE id = $1
                    RETURNING id, mission_id, rover_id, status, progress, waypoint, lap, error, created_at, started_at, ended_at
                    "#,
                )
                .bind(task_id)
                .bind(progress)
                .bind(waypoint)
                .bind(lap)
                .fetch_one(&state.db)
                .await;

                if let Ok(task) = result {
                    state.broadcast(BroadcastMessage::TaskUpdate { task });
                }
            }
            Ok(RoverToDispatch::Complete { task_id, laps }) => {
                info!(task_id = %task_id, laps = laps, "Task completed");

                let result: Result<Task, _> = sqlx::query_as(
                    r#"
                    UPDATE tasks 
                    SET status = 'done', progress = 100, lap = $2, ended_at = now()
                    WHERE id = $1
                    RETURNING id, mission_id, rover_id, status, progress, waypoint, lap, error, created_at, started_at, ended_at
                    "#,
                )
                .bind(task_id)
                .bind(laps)
                .fetch_one(&state.db)
                .await;

                if let Ok(task) = result {
                    let rid = task.rover_id.clone();

                    // Clear rover's current task
                    {
                        let mut rovers = state.rovers.write().await;
                        if let Some(rover) = rovers.get_mut(&rid) {
                            rover.current_task = None;
                        }
                    }

                    state.broadcast(BroadcastMessage::TaskUpdate { task });
                    state.broadcast(BroadcastMessage::RoverUpdate {
                        rover_id: rid,
                        connected: true,
                        task_id: None,
                    });
                }
            }
            Ok(RoverToDispatch::Failed { task_id, error }) => {
                warn!(task_id = %task_id, error = %error, "Task failed");

                let result: Result<Task, _> = sqlx::query_as(
                    r#"
                    UPDATE tasks 
                    SET status = 'failed', error = $2, ended_at = now()
                    WHERE id = $1
                    RETURNING id, mission_id, rover_id, status, progress, waypoint, lap, error, created_at, started_at, ended_at
                    "#,
                )
                .bind(task_id)
                .bind(&error)
                .fetch_one(&state.db)
                .await;

                if let Ok(task) = result {
                    let rid = task.rover_id.clone();

                    // Clear rover's current task
                    {
                        let mut rovers = state.rovers.write().await;
                        if let Some(rover) = rovers.get_mut(&rid) {
                            rover.current_task = None;
                        }
                    }

                    state.broadcast(BroadcastMessage::TaskUpdate { task });
                    state.broadcast(BroadcastMessage::RoverUpdate {
                        rover_id: rid,
                        connected: true,
                        task_id: None,
                    });
                }
            }
            Err(e) => {
                warn!(error = %e, "Failed to parse rover message");
            }
        }
    }

    // Cleanup on disconnect
    if let Some(id) = rover_id {
        info!(rover_id = %id, "Rover disconnected");
        let mut rovers = state.rovers.write().await;
        rovers.remove(&id);
        drop(rovers);

        state.broadcast(BroadcastMessage::RoverUpdate {
            rover_id: id,
            connected: false,
            task_id: None,
        });
    }

    send_task.abort();
}

/// WebSocket handler for console clients
async fn ws_console_handler(
    ws: WebSocketUpgrade,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_console_ws(socket, state))
}

async fn handle_console_ws(socket: WebSocket, state: SharedState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.broadcast_tx.subscribe();

    info!("Console client connected");

    // Send initial state
    let rovers = state.rovers.read().await;
    for (rover_id, rover) in rovers.iter() {
        let msg = BroadcastMessage::RoverUpdate {
            rover_id: rover_id.clone(),
            connected: true,
            task_id: rover.current_task,
        };
        if let Ok(json) = serde_json::to_string(&msg) {
            let _ = sender.send(Message::Text(json.into())).await;
        }
    }
    drop(rovers);

    loop {
        tokio::select! {
            Ok(msg) = rx.recv() => {
                if let Ok(json) = serde_json::to_string(&msg) {
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
            }
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(data))) => {
                        let _ = sender.send(Message::Pong(data)).await;
                    }
                    _ => {}
                }
            }
        }
    }

    info!("Console client disconnected");
}
