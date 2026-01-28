//! Server-side alert evaluation, persistence, and API.
//!
//! Replaces client-side alert derivation with persistent, server-authoritative alerts.
//! Alerts are stored in PostgreSQL and broadcast to console clients via WebSocket.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use tracing::info;
use uuid::Uuid;

use crate::SharedState;

// =============================================================================
// Types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    pub id: Uuid,
    pub severity: String,
    pub source: String,
    pub source_id: String,
    pub message: String,
    pub created_at: DateTime<Utc>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub acknowledged_by: Option<String>,
    pub cleared_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct AlertsQuery {
    pub severity: Option<String>,
    pub source: Option<String>,
    pub active: Option<bool>,
    pub since: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct AckRequest {
    pub by: Option<String>,
}

/// Broadcast message types for alerts
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AlertBroadcast {
    AlertCreated { alert: Alert },
    AlertAcknowledged { id: Uuid, by: String },
    AlertCleared { id: Uuid },
}

// =============================================================================
// Database Operations
// =============================================================================

/// Create a new alert and return it
pub async fn create_alert(
    pool: &PgPool,
    severity: &str,
    source: &str,
    source_id: &str,
    message: &str,
    metadata: Option<serde_json::Value>,
) -> Result<Alert, sqlx::Error> {
    let meta = metadata.unwrap_or(serde_json::json!({}));
    sqlx::query_as(
        r#"
        INSERT INTO alerts (severity, source, source_id, message, metadata)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, severity, source, source_id, message, created_at,
                  acknowledged_at, acknowledged_by, cleared_at, metadata
        "#,
    )
    .bind(severity)
    .bind(source)
    .bind(source_id)
    .bind(message)
    .bind(&meta)
    .fetch_one(pool)
    .await
}

/// Check if an active (uncleared) alert already exists with the same source+source_id+message
pub async fn has_active_alert(
    pool: &PgPool,
    source: &str,
    source_id: &str,
    message: &str,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM alerts
            WHERE cleared_at IS NULL
              AND source = $1
              AND source_id = $2
              AND message = $3
        )
        "#,
    )
    .bind(source)
    .bind(source_id)
    .bind(message)
    .fetch_one(pool)
    .await
}

/// Clear all active alerts matching source+source_id+message pattern
pub async fn clear_alerts_matching(
    pool: &PgPool,
    source: &str,
    source_id: &str,
    message_pattern: &str,
) -> Result<Vec<Uuid>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        UPDATE alerts
        SET cleared_at = now()
        WHERE cleared_at IS NULL
          AND source = $1
          AND source_id = $2
          AND message LIKE $3
        RETURNING id
        "#,
    )
    .bind(source)
    .bind(source_id)
    .bind(message_pattern)
    .fetch_all(pool)
    .await
}

// =============================================================================
// REST Endpoints
// =============================================================================

/// GET /api/alerts — list alerts with optional filters
pub async fn list_alerts(
    State(state): State<SharedState>,
    Query(query): Query<AlertsQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(200);
    let active_only = query.active.unwrap_or(false);

    let alerts: Vec<Alert> = if let Some(ref severity) = query.severity {
        if active_only {
            sqlx::query_as(
                r#"
                SELECT id, severity, source, source_id, message, created_at,
                       acknowledged_at, acknowledged_by, cleared_at, metadata
                FROM alerts
                WHERE severity = $1 AND cleared_at IS NULL
                ORDER BY created_at DESC LIMIT $2
                "#,
            )
            .bind(severity)
            .bind(limit)
            .fetch_all(&state.db)
            .await
        } else {
            sqlx::query_as(
                r#"
                SELECT id, severity, source, source_id, message, created_at,
                       acknowledged_at, acknowledged_by, cleared_at, metadata
                FROM alerts
                WHERE severity = $1
                ORDER BY created_at DESC LIMIT $2
                "#,
            )
            .bind(severity)
            .bind(limit)
            .fetch_all(&state.db)
            .await
        }
    } else if let Some(ref source) = query.source {
        sqlx::query_as(
            r#"
            SELECT id, severity, source, source_id, message, created_at,
                   acknowledged_at, acknowledged_by, cleared_at, metadata
            FROM alerts
            WHERE source = $1
            ORDER BY created_at DESC LIMIT $2
            "#,
        )
        .bind(source)
        .bind(limit)
        .fetch_all(&state.db)
        .await
    } else if let Some(since_ms) = query.since {
        let since = DateTime::from_timestamp_millis(since_ms)
            .unwrap_or_else(|| Utc::now());
        sqlx::query_as(
            r#"
            SELECT id, severity, source, source_id, message, created_at,
                   acknowledged_at, acknowledged_by, cleared_at, metadata
            FROM alerts
            WHERE created_at >= $1
            ORDER BY created_at DESC LIMIT $2
            "#,
        )
        .bind(since)
        .bind(limit)
        .fetch_all(&state.db)
        .await
    } else if active_only {
        sqlx::query_as(
            r#"
            SELECT id, severity, source, source_id, message, created_at,
                   acknowledged_at, acknowledged_by, cleared_at, metadata
            FROM alerts
            WHERE cleared_at IS NULL
            ORDER BY created_at DESC LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query_as(
            r#"
            SELECT id, severity, source, source_id, message, created_at,
                   acknowledged_at, acknowledged_by, cleared_at, metadata
            FROM alerts
            ORDER BY created_at DESC LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&state.db)
        .await
    }
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(alerts))
}

/// POST /api/alerts/:id/ack — acknowledge an alert
pub async fn acknowledge_alert(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(body): Json<AckRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let by = body.by.unwrap_or_else(|| "operator".to_string());

    let alert: Alert = sqlx::query_as(
        r#"
        UPDATE alerts
        SET acknowledged_at = now(), acknowledged_by = $2
        WHERE id = $1 AND acknowledged_at IS NULL
        RETURNING id, severity, source, source_id, message, created_at,
                  acknowledged_at, acknowledged_by, cleared_at, metadata
        "#,
    )
    .bind(id)
    .bind(&by)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Alert not found or already acknowledged".to_string()))?;

    info!(id = %id, by = %by, "Alert acknowledged");

    // Broadcast
    let msg = AlertBroadcast::AlertAcknowledged { id, by };
    broadcast_alert(&state, msg);

    Ok(Json(alert))
}

/// POST /api/alerts/:id/clear — clear an alert
pub async fn clear_alert(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let alert: Alert = sqlx::query_as(
        r#"
        UPDATE alerts
        SET cleared_at = now()
        WHERE id = $1 AND cleared_at IS NULL
        RETURNING id, severity, source, source_id, message, created_at,
                  acknowledged_at, acknowledged_by, cleared_at, metadata
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Alert not found or already cleared".to_string()))?;

    info!(id = %id, "Alert cleared");

    let msg = AlertBroadcast::AlertCleared { id };
    broadcast_alert(&state, msg);

    Ok(Json(alert))
}

/// DELETE /api/alerts/:id — delete an alert (admin)
pub async fn delete_alert(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM alerts WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Alert not found".to_string()));
    }

    info!(id = %id, "Alert deleted");
    Ok(StatusCode::NO_CONTENT)
}

// =============================================================================
// Alert Evaluation Loop
// =============================================================================

/// Background loop that evaluates rover state and creates/clears alerts.
/// Runs every second, comparing current rover state with previous snapshots.
pub async fn alert_evaluator(state: SharedState) {
    use std::collections::HashMap;
    use std::time::Duration;
    use tokio::time::interval;

    let mut ticker = interval(Duration::from_secs(2));
    let mut prev_rovers: HashMap<String, RoverSnapshot> = HashMap::new();

    info!("Starting alert evaluation loop");

    loop {
        ticker.tick().await;

        // Get connected rover info from discovery (via dispatch state)
        let rovers = state.rovers.read().await;
        let connected_ids: Vec<String> = rovers.keys().cloned().collect();
        drop(rovers);

        // Check for disconnected rovers that were previously connected
        let mut to_remove = Vec::new();
        for (rover_id, _snapshot) in &prev_rovers {
            if !connected_ids.contains(rover_id) {
                // Rover disconnected — create alert if not already active
                if let Ok(false) = has_active_alert(
                    &state.db, "rover", rover_id, "Connection lost",
                ).await {
                    if let Ok(alert) = create_alert(
                        &state.db, "critical", "rover", rover_id,
                        "Connection lost", None,
                    ).await {
                        state.webhook.notify(&alert);
                        broadcast_alert(&state, AlertBroadcast::AlertCreated { alert });
                    }
                }
                to_remove.push(rover_id.clone());
            }
        }

        for id in to_remove {
            prev_rovers.remove(&id);
        }

        // Track newly connected rovers
        for rover_id in &connected_ids {
            if !prev_rovers.contains_key(rover_id) {
                // Rover just connected — clear any "Connection lost" alerts
                if let Ok(cleared_ids) = clear_alerts_matching(
                    &state.db, "rover", rover_id, "Connection lost",
                ).await {
                    for cid in cleared_ids {
                        broadcast_alert(&state, AlertBroadcast::AlertCleared { id: cid });
                    }
                }

                // Create info alert for connection (info-level, webhook skips it)
                if let Ok(alert) = create_alert(
                    &state.db, "info", "rover", rover_id,
                    "Rover connected to dispatch", None,
                ).await {
                    broadcast_alert(&state, AlertBroadcast::AlertCreated { alert });
                }

                prev_rovers.insert(rover_id.clone(), RoverSnapshot {
                    connected: true,
                });
            }
        }

        // Check for failed tasks and create alerts
        let failed_tasks: Vec<(Uuid, String, String)> = sqlx::query_as(
            r#"
            SELECT t.id, t.rover_id, COALESCE(t.error, 'Unknown error')
            FROM tasks t
            LEFT JOIN alerts a ON a.source = 'dispatch'
              AND a.source_id = t.id::text
              AND a.cleared_at IS NULL
            WHERE t.status = 'failed'
              AND t.ended_at > now() - interval '5 minutes'
              AND a.id IS NULL
            "#,
        )
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

        for (task_id, rover_id, error) in failed_tasks {
            let message = format!("Task failed on {}: {}", rover_id, error);
            if let Ok(alert) = create_alert(
                &state.db, "warning", "dispatch", &task_id.to_string(),
                &message, None,
            ).await {
                state.webhook.notify(&alert);
                broadcast_alert(&state, AlertBroadcast::AlertCreated { alert });
            }
        }

        // Auto-clear old alerts (older than 24 hours)
        let _ = sqlx::query(
            "UPDATE alerts SET cleared_at = now() WHERE cleared_at IS NULL AND created_at < now() - interval '24 hours'"
        )
        .execute(&state.db)
        .await;
    }
}

// =============================================================================
// Internal Helpers
// =============================================================================

#[derive(Debug, Clone)]
struct RoverSnapshot {
    connected: bool,
}

pub(crate) fn broadcast_alert(state: &SharedState, msg: AlertBroadcast) {
    if let Ok(json) = serde_json::to_string(&msg) {
        // Use the existing broadcast channel — serialize as a raw JSON message
        // that the console WebSocket handler will forward
        let _ = state.broadcast_tx.send(crate::BroadcastMessage::AlertEvent { json });
    }
}
