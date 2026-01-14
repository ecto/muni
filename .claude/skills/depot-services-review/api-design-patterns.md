# API Design Patterns

Comprehensive guide for Axum web framework patterns in depot services.

## Overview

All depot services use **Axum 0.7-0.8** as the web framework, built on top of Tokio and Hyper.

**Why Axum?**
- Ergonomic routing with extractors
- Type-safe request handling
- Excellent async/await support
- Built on battle-tested Hyper
- Great error handling with `IntoResponse`
- WebSocket support built-in

## Routing

### Basic Router Setup

```rust
use axum::{
    routing::{get, post, put, delete},
    Router,
};

#[tokio::main]
async fn main() {
    let app = Router::new()
        // Health check
        .route("/health", get(health))
        // CRUD endpoints
        .route("/zones", post(create_zone))
        .route("/zones", get(list_zones))
        .route("/zones/{id}", get(get_zone))
        .route("/zones/{id}", put(update_zone))
        .route("/zones/{id}", delete(delete_zone))
        // WebSocket
        .route("/ws", get(ws_handler))
        // Attach state
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4860").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### RESTful Route Patterns

**Resource-based routing**:
```rust
// Collections (plural noun)
.route("/zones", post(create_zone))      // POST /zones - create
.route("/zones", get(list_zones))        // GET /zones - list all

// Individual resources (with ID)
.route("/zones/{id}", get(get_zone))     // GET /zones/:id - get one
.route("/zones/{id}", put(update_zone))  // PUT /zones/:id - update
.route("/zones/{id}", delete(delete_zone))  // DELETE /zones/:id - delete

// Nested resources
.route("/zones/{id}/missions", get(list_zone_missions))  // GET /zones/:id/missions

// Actions (verbs for non-CRUD operations)
.route("/missions/{id}/start", post(start_mission))  // POST /missions/:id/start
.route("/missions/{id}/stop", post(stop_mission))    // POST /missions/:id/stop
```

### Route Grouping

```rust
// Group related routes with nested routers
let zones_router = Router::new()
    .route("/", post(create_zone))
    .route("/", get(list_zones))
    .route("/{id}", get(get_zone))
    .route("/{id}", put(update_zone))
    .route("/{id}", delete(delete_zone));

let missions_router = Router::new()
    .route("/", post(create_mission))
    .route("/", get(list_missions))
    .route("/{id}/start", post(start_mission))
    .route("/{id}/stop", post(stop_mission));

let app = Router::new()
    .nest("/zones", zones_router)
    .nest("/missions", missions_router)
    .route("/health", get(health))
    .with_state(state);
```

## Handlers

### Handler Function Signature

```rust
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

async fn handler_name(
    // Extractors (order doesn't matter)
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
    Query(params): Query<QueryParams>,
    Json(payload): Json<RequestPayload>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Handler logic
    Ok((StatusCode::OK, Json(response_data)))
}
```

**Return types**:
- `impl IntoResponse` - any type that can be converted to HTTP response
- `Result<impl IntoResponse, E>` where `E: IntoResponse` - for error handling
- Common response types: `Json<T>`, `StatusCode`, `(StatusCode, Json<T>)`, `String`, `Vec<u8>`

### Extractors

**Path parameters**:
```rust
// Route: /zones/{id}
async fn get_zone(
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Use `id` here
}

// Multiple path parameters
// Route: /zones/{zone_id}/missions/{mission_id}
async fn get_zone_mission(
    Path((zone_id, mission_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Use `zone_id` and `mission_id`
}
```

**Query parameters**:
```rust
#[derive(Deserialize)]
struct ListQuery {
    status: Option<String>,
    rover_id: Option<String>,
    limit: Option<u32>,
}

// Route: /tasks?status=active&rover_id=rover1&limit=10
async fn list_tasks(
    Query(query): Query<ListQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let status = query.status;  // Option<String>
    let rover_id = query.rover_id;  // Option<String>
    let limit = query.limit.unwrap_or(100);  // Default to 100
}
```

**Request body (JSON)**:
```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateZoneRequest {
    name: String,
    zone_type: String,
    waypoints: Vec<Waypoint>,
}

async fn create_zone(
    Json(payload): Json<CreateZoneRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Use `payload.name`, `payload.zone_type`, etc.
}
```

**Application state**:
```rust
type SharedState = Arc<AppState>;

async fn handler(
    State(state): State<SharedState>,
) -> impl IntoResponse {
    // Access shared state
    let count = state.rovers.read().await.len();
    Json(serde_json::json!({ "count": count }))
}
```

### Response Types

**JSON response**:
```rust
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ZoneResponse {
    id: Uuid,
    name: String,
    created_at: DateTime<Utc>,
}

async fn get_zone() -> impl IntoResponse {
    let zone = ZoneResponse {
        id: Uuid::new_v4(),
        name: "Test Zone".to_string(),
        created_at: Utc::now(),
    };
    Json(zone)  // Automatically sets Content-Type: application/json
}
```

**Status code with JSON**:
```rust
async fn create_zone() -> impl IntoResponse {
    let zone = create_zone_in_db().await;
    (StatusCode::CREATED, Json(zone))  // 201 Created
}
```

**Status code only**:
```rust
async fn delete_zone() -> impl IntoResponse {
    delete_zone_from_db().await;
    StatusCode::NO_CONTENT  // 204 No Content (no body)
}
```

**Custom headers**:
```rust
use axum::http::header;

async fn get_asset() -> impl IntoResponse {
    let content = read_file().await;
    (
        [
            (header::CONTENT_TYPE, "application/octet-stream"),
            (header::CONTENT_DISPOSITION, "attachment; filename=\"file.bin\""),
        ],
        content,
    )
}
```

**Text response**:
```rust
async fn handler() -> impl IntoResponse {
    "Hello, world!"  // Plain text
}
```

**Binary response**:
```rust
async fn get_image() -> impl IntoResponse {
    let image_bytes: Vec<u8> = load_image().await;
    (
        [(header::CONTENT_TYPE, "image/jpeg")],
        image_bytes,
    )
}
```

## Error Handling

### Result-Based Error Handling

**Simple tuple errors**:
```rust
async fn handler() -> Result<impl IntoResponse, (StatusCode, String)> {
    let data = fetch_data()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if data.is_empty() {
        return Err((StatusCode::NOT_FOUND, "No data found".to_string()));
    }

    Ok(Json(data))
}
```

**Custom error types**:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            ApiError::Io(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            ApiError::Database(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            ApiError::Json(e) => (StatusCode::BAD_REQUEST, e.to_string()),
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

// Usage
async fn handler() -> Result<impl IntoResponse, ApiError> {
    let file = tokio::fs::read("file.txt").await?;  // Auto-converts io::Error
    let data: MyData = serde_json::from_slice(&file)?;  // Auto-converts serde_json::Error
    Ok(Json(data))
}
```

### HTTP Status Codes

**Success codes**:
- `200 OK` - Successful GET, PUT, PATCH
- `201 CREATED` - Successful POST creating a resource
- `204 NO_CONTENT` - Successful DELETE, no response body

**Client error codes**:
- `400 BAD_REQUEST` - Invalid request payload, malformed JSON
- `404 NOT_FOUND` - Resource doesn't exist
- `409 CONFLICT` - Resource conflict (e.g., duplicate, concurrent modification)
- `422 UNPROCESSABLE_ENTITY` - Valid JSON but semantically invalid

**Server error codes**:
- `500 INTERNAL_SERVER_ERROR` - Unexpected error
- `503 SERVICE_UNAVAILABLE` - Service temporarily unavailable (e.g., database down)

```rust
async fn create_zone(
    State(state): State<SharedState>,
    Json(payload): Json<CreateZone>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Validation
    if payload.name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Name cannot be empty".to_string()));
    }

    // Check for duplicate
    let exists = check_zone_exists(&payload.name).await?;
    if exists {
        return Err((StatusCode::CONFLICT, "Zone with this name already exists".to_string()));
    }

    // Create zone
    let zone = sqlx::query_as("INSERT INTO zones ...")
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(zone)))
}
```

## Middleware

### CORS (Cross-Origin Resource Sharing)

```rust
use tower_http::cors::CorsLayer;

let app = Router::new()
    .route("/endpoint", get(handler))
    .layer(CorsLayer::permissive());  // Allow all origins for internal services
```

**Custom CORS**:
```rust
use tower_http::cors::{Any, CorsLayer};
use axum::http::Method;

let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers(Any);

let app = Router::new()
    .route("/endpoint", get(handler))
    .layer(cors);
```

### Request Tracing

```rust
use tower_http::trace::TraceLayer;

let app = Router::new()
    .route("/endpoint", get(handler))
    .layer(TraceLayer::new_for_http());  // Logs all requests
```

### Compression

```rust
use tower_http::compression::CompressionLayer;

let app = Router::new()
    .route("/endpoint", get(handler))
    .layer(CompressionLayer::new());  // Compress responses
```

### Timeout

```rust
use tower::timeout::TimeoutLayer;
use std::time::Duration;

let app = Router::new()
    .route("/endpoint", get(handler))
    .layer(TimeoutLayer::new(Duration::from_secs(30)));
```

## WebSocket

### Basic WebSocket Handler

```rust
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_ws(socket, state))
}

async fn handle_ws(socket: WebSocket, state: SharedState) {
    let (mut sender, mut receiver) = socket.split();

    // Send initial message
    let _ = sender.send(Message::Text("Connected".into())).await;

    // Receive messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                info!("Received: {}", text);
                // Process message
                let response = process_message(&text);
                let _ = sender.send(Message::Text(response.into())).await;
            }
            Ok(Message::Close(_)) => {
                info!("Client closed connection");
                break;
            }
            Ok(Message::Ping(data)) => {
                // Pong is handled automatically by Axum
            }
            _ => {}
        }
    }

    info!("WebSocket connection closed");
}
```

### WebSocket with Protocol

```rust
use serde::{Deserialize, Serialize};

// Client -> Server messages
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum ClientMessage {
    Register { rover_id: String },
    Progress { task_id: Uuid, progress: i32 },
}

// Server -> Client messages
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum ServerMessage {
    Task { task_id: Uuid, waypoints: Vec<Waypoint> },
    Cancel { task_id: Uuid },
}

async fn handle_ws(socket: WebSocket, state: SharedState) {
    let (mut sender, mut receiver) = socket.split();

    while let Some(msg) = receiver.next().await {
        let msg = match msg {
            Ok(Message::Text(text)) => text,
            Ok(Message::Close(_)) => break,
            _ => continue,
        };

        // Parse message
        let parsed: Result<ClientMessage, _> = serde_json::from_str(&msg);
        match parsed {
            Ok(ClientMessage::Register { rover_id }) => {
                info!(rover_id = %rover_id, "Rover registered");
                // Handle registration
            }
            Ok(ClientMessage::Progress { task_id, progress }) => {
                info!(task_id = %task_id, progress = progress, "Task progress");
                // Handle progress update
            }
            Err(e) => {
                warn!(error = %e, "Failed to parse message");
            }
        }
    }
}
```

### WebSocket with Broadcast

```rust
use tokio::sync::broadcast;

struct AppState {
    broadcast_tx: broadcast::Sender<ServerMessage>,
}

async fn handle_ws(socket: WebSocket, state: SharedState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.broadcast_tx.subscribe();

    loop {
        tokio::select! {
            // Receive from broadcast channel
            Ok(msg) = rx.recv() => {
                if let Ok(json) = serde_json::to_string(&msg) {
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
            }

            // Receive from WebSocket
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        // Handle incoming message
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
}
```

**Reference**: See [websocket-patterns.md](./websocket-patterns.md) for detailed WebSocket guidance.

## Request Validation

### Input Validation

```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateZone {
    name: String,
    zone_type: String,
    waypoints: Vec<Waypoint>,
}

async fn create_zone(
    Json(payload): Json<CreateZone>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Validate name
    if payload.name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Name cannot be empty".to_string()));
    }
    if payload.name.len() > 100 {
        return Err((StatusCode::BAD_REQUEST, "Name too long (max 100 chars)".to_string()));
    }

    // Validate zone type
    if !["route", "polygon", "point"].contains(&payload.zone_type.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "Invalid zone type".to_string()));
    }

    // Validate waypoints
    if payload.waypoints.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "At least one waypoint required".to_string()));
    }

    // Proceed with creation
    Ok(Json(create_zone_in_db(payload).await?))
}
```

### JSON Schema Validation

For complex validation, consider using `jsonschema` or `validator` crates:

```toml
[dependencies]
validator = { version = "0.16", features = ["derive"] }
```

```rust
use validator::Validate;

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
struct CreateZone {
    #[validate(length(min = 1, max = 100))]
    name: String,

    #[validate(custom = "validate_zone_type")]
    zone_type: String,

    #[validate(length(min = 1))]
    waypoints: Vec<Waypoint>,
}

fn validate_zone_type(zone_type: &str) -> Result<(), validator::ValidationError> {
    if ["route", "polygon", "point"].contains(&zone_type) {
        Ok(())
    } else {
        Err(validator::ValidationError::new("invalid_zone_type"))
    }
}

async fn create_zone(
    Json(payload): Json<CreateZone>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Validate
    payload.validate()
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Validation error: {}", e)))?;

    // Proceed with creation
    Ok(Json(create_zone_in_db(payload).await?))
}
```

## Testing

### Handler Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;  // for `oneshot`

    #[tokio::test]
    async fn test_health_endpoint() {
        let state = Arc::new(AppState::new());
        let app = Router::new()
            .route("/health", get(health))
            .with_state(state);

        let response = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_create_zone() {
        let state = Arc::new(AppState::new());
        let app = Router::new()
            .route("/zones", post(create_zone))
            .with_state(state);

        let payload = serde_json::json!({
            "name": "Test Zone",
            "zoneType": "route",
            "waypoints": [{"x": 0.0, "y": 0.0}]
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/zones")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }
}
```

## Performance Best Practices

### Avoid Blocking Operations

❌ BAD:
```rust
async fn handler() -> impl IntoResponse {
    let data = std::fs::read_to_string("file.txt").unwrap();  // Blocking I/O!
    Json(data)
}
```

✅ GOOD:
```rust
async fn handler() -> impl IntoResponse {
    let data = tokio::fs::read_to_string("file.txt").await.unwrap();  // Async I/O
    Json(data)
}
```

### Use Streaming for Large Responses

```rust
use axum::body::Body;
use tokio_util::io::ReaderStream;

async fn serve_large_file() -> impl IntoResponse {
    let file = tokio::fs::File::open("large_file.bin").await.unwrap();
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    (
        [(header::CONTENT_TYPE, "application/octet-stream")],
        body,
    )
}
```

### Limit Request Body Size

```rust
use axum::extract::DefaultBodyLimit;
use tower::ServiceBuilder;

let app = Router::new()
    .route("/upload", post(upload_handler))
    .layer(
        ServiceBuilder::new()
            .layer(DefaultBodyLimit::max(10 * 1024 * 1024))  // 10 MB limit
    );
```

## Common Patterns

### Paginated Responses

```rust
#[derive(Deserialize)]
struct PaginationQuery {
    page: Option<u32>,
    per_page: Option<u32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PaginatedResponse<T> {
    data: Vec<T>,
    total: u32,
    page: u32,
    per_page: u32,
    total_pages: u32,
}

async fn list_zones(
    Query(query): Query<PaginationQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM zones")
        .fetch_one(&pool)
        .await?;

    let zones: Vec<Zone> = sqlx::query_as(
        "SELECT * FROM zones ORDER BY created_at DESC LIMIT $1 OFFSET $2"
    )
    .bind(per_page as i64)
    .bind(offset as i64)
    .fetch_all(&pool)
    .await?;

    let total_pages = (total as f64 / per_page as f64).ceil() as u32;

    Ok(Json(PaginatedResponse {
        data: zones,
        total: total as u32,
        page,
        per_page,
        total_pages,
    }))
}
```

### File Upload

```rust
use axum::extract::Multipart;

async fn upload_file(
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        info!(name = %name, size = data.len(), "Received file");

        // Save file
        tokio::fs::write(format!("uploads/{}", name), data)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    Ok(StatusCode::OK)
}
```
