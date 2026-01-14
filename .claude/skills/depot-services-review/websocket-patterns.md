# WebSocket Patterns

Comprehensive guide for WebSocket communication in depot services.

## Overview

Three depot services use WebSockets:
- **Discovery**: Broadcasts rover status updates to console
- **Dispatch**: Bidirectional communication with rovers and console
- **GPS-Status**: Broadcasts RTK base station status to console

**Technology**:
- Axum WebSocket support (built-in)
- futures-util for Stream/Sink utilities
- tokio::sync::broadcast for fan-out messaging

## Basic WebSocket Setup

### Handler Registration

```rust
use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};

let app = Router::new()
    .route("/ws", get(ws_handler))  // WebSocket always uses GET
    .with_state(state);
```

### WebSocket Upgrade Handler

```rust
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    // on_upgrade takes async function that receives the socket
    ws.on_upgrade(|socket| handle_ws(socket, state))
}
```

### WebSocket Connection Handler

```rust
use futures_util::{SinkExt, StreamExt};
use axum::extract::ws::Message;

async fn handle_ws(socket: WebSocket, state: SharedState) {
    // Split socket into sender and receiver
    let (mut sender, mut receiver) = socket.split();

    info!("Client connected");

    // Receive messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                info!("Received: {}", text);
                // Process message and send response
                let response = process_message(&text);
                let _ = sender.send(Message::Text(response.into())).await;
            }
            Ok(Message::Close(frame)) => {
                info!("Client closed connection: {:?}", frame);
                break;
            }
            Ok(Message::Ping(data)) => {
                // Pong is automatically sent by Axum
                debug!("Received ping");
            }
            Ok(Message::Pong(_)) => {
                // Response to our ping
                debug!("Received pong");
            }
            Ok(Message::Binary(_)) => {
                warn!("Received binary message (not supported)");
            }
            Err(e) => {
                warn!(error = %e, "WebSocket error");
                break;
            }
        }
    }

    info!("Client disconnected");
    // Cleanup happens here
}
```

## Message Protocols

### JSON Message Protocol

**Define message types**:
```rust
use serde::{Deserialize, Serialize};

// Client -> Server messages
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ClientMessage {
    Register {
        rover_id: String,
    },
    Progress {
        task_id: Uuid,
        progress: i32,
        waypoint: i32,
    },
    Complete {
        task_id: Uuid,
    },
}

// Server -> Client messages
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ServerMessage {
    Task {
        task_id: Uuid,
        waypoints: Vec<Waypoint>,
    },
    Cancel {
        task_id: Uuid,
    },
    Status {
        online: bool,
    },
}
```

**Parse incoming messages**:
```rust
async fn handle_ws(socket: WebSocket, state: SharedState) {
    let (mut sender, mut receiver) = socket.split();

    while let Some(msg) = receiver.next().await {
        let text = match msg {
            Ok(Message::Text(t)) => t,
            Ok(Message::Close(_)) => break,
            _ => continue,
        };

        // Parse JSON
        match serde_json::from_str::<ClientMessage>(&text) {
            Ok(ClientMessage::Register { rover_id }) => {
                info!(rover_id = %rover_id, "Rover registered");
                // Handle registration
            }
            Ok(ClientMessage::Progress { task_id, progress, waypoint }) => {
                info!(task_id = %task_id, progress, "Task progress");
                // Update task in database
            }
            Ok(ClientMessage::Complete { task_id }) => {
                info!(task_id = %task_id, "Task completed");
                // Mark task as complete
            }
            Err(e) => {
                warn!(error = %e, message = %text, "Failed to parse message");
            }
        }
    }
}
```

**Send messages**:
```rust
// Serialize and send
let msg = ServerMessage::Task {
    task_id: Uuid::new_v4(),
    waypoints: vec![...],
};

if let Ok(json) = serde_json::to_string(&msg) {
    if let Err(e) = sender.send(Message::Text(json.into())).await {
        warn!(error = %e, "Failed to send message");
    }
}
```

## Broadcast Pattern

### Setup Broadcast Channel

```rust
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize)]
pub enum BroadcastMessage {
    RoverUpdate { rover_id: String, online: bool },
    TaskUpdate { task_id: Uuid, status: String },
}

struct AppState {
    broadcast_tx: broadcast::Sender<BroadcastMessage>,
}

impl AppState {
    fn new() -> Self {
        // Create broadcast channel with capacity 256
        let (broadcast_tx, _rx) = broadcast::channel(256);
        Self { broadcast_tx }
    }

    // Helper to broadcast messages
    fn broadcast(&self, msg: BroadcastMessage) {
        // Ignore error if no receivers
        let _ = self.broadcast_tx.send(msg);
    }
}
```

**Why drop the receiver `_rx`?**
- Broadcast channels need at least one receiver to exist
- We create it but immediately drop it
- Each WebSocket subscribes with `.subscribe()`

### WebSocket with Broadcast

```rust
async fn handle_ws(socket: WebSocket, state: SharedState) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to broadcast channel
    let mut rx = state.broadcast_tx.subscribe();

    // Send initial state
    let initial_state = get_initial_state(&state).await;
    if let Ok(json) = serde_json::to_string(&initial_state) {
        let _ = sender.send(Message::Text(json.into())).await;
    }

    info!("Client connected");

    loop {
        tokio::select! {
            // Receive from broadcast channel
            Ok(msg) = rx.recv() => {
                if let Ok(json) = serde_json::to_string(&msg) {
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        break;  // Client disconnected
                    }
                }
            }

            // Receive from WebSocket
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        // Handle incoming message
                        handle_client_message(&text, &state).await;
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        break;
                    }
                    Some(Ok(Message::Ping(_))) => {
                        // Auto-handled by Axum
                    }
                    _ => {}
                }
            }
        }
    }

    info!("Client disconnected");
}
```

**Key points**:
- Use `tokio::select!` to wait on multiple async operations
- Broadcast messages are sent to all connected clients
- If client disconnects, the loop breaks and cleanup happens
- Lagged receivers are automatically handled by broadcast channel

### Handling Lagged Receivers

```rust
loop {
    tokio::select! {
        result = rx.recv() => {
            match result {
                Ok(msg) => {
                    // Send to client
                    if let Ok(json) = serde_json::to_string(&msg) {
                        if sender.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    // Client is too slow, missed n messages
                    warn!(lagged = n, "Client lagged, resending full state");
                    // Send full state to recover
                    send_full_state(&mut sender, &state).await;
                }
                Err(broadcast::error::RecvError::Closed) => {
                    // Broadcast channel closed
                    break;
                }
            }
        }
        // ... other select branches
    }
}
```

## Per-Client State

### Tracking Connected Clients

```rust
struct ConnectedClient {
    id: String,
    tx: tokio::sync::mpsc::Sender<ServerMessage>,
}

struct AppState {
    clients: RwLock<HashMap<String, ConnectedClient>>,
    broadcast_tx: broadcast::Sender<BroadcastMessage>,
}

async fn handle_ws(socket: WebSocket, state: SharedState) {
    let (mut sender, mut receiver) = socket.split();

    // Create channel for sending messages to this client
    let (tx, mut rx) = tokio::sync::mpsc::channel::<ServerMessage>(32);

    let mut client_id: Option<String> = None;

    // Spawn task to forward messages to client
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
        let text = match msg {
            Ok(Message::Text(t)) => t,
            Ok(Message::Close(_)) => break,
            _ => continue,
        };

        match serde_json::from_str::<ClientMessage>(&text) {
            Ok(ClientMessage::Register { rover_id }) => {
                client_id = Some(rover_id.clone());

                // Register client
                let mut clients = state.clients.write().await;
                clients.insert(rover_id.clone(), ConnectedClient {
                    id: rover_id.clone(),
                    tx: tx.clone(),
                });
                drop(clients);

                info!(client_id = %rover_id, "Client registered");
            }
            // ... handle other messages
            _ => {}
        }
    }

    // Cleanup on disconnect
    if let Some(id) = client_id {
        info!(client_id = %id, "Client disconnected");
        let mut clients = state.clients.write().await;
        clients.remove(&id);
    }

    send_task.abort();
}
```

### Sending to Specific Client

```rust
impl AppState {
    pub async fn send_to_client(&self, client_id: &str, msg: ServerMessage) -> bool {
        let clients = self.clients.read().await;
        if let Some(client) = clients.get(client_id) {
            client.tx.send(msg).await.is_ok()
        } else {
            false
        }
    }
}

// Usage
let sent = state.send_to_client("rover1", ServerMessage::Task { ... }).await;
if !sent {
    warn!("Failed to send message to rover1");
}
```

## Error Handling

### Connection Errors

```rust
async fn handle_ws(socket: WebSocket, state: SharedState) {
    let (mut sender, mut receiver) = socket.split();

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Process message
            }
            Ok(Message::Close(frame)) => {
                info!("Client closed: {:?}", frame);
                break;
            }
            Err(e) => {
                warn!(error = %e, "WebSocket error");
                break;
            }
            _ => {}
        }
    }

    // Always do cleanup
    cleanup_client(&state).await;
}
```

### Send Errors

```rust
// Check if send succeeds
if sender.send(Message::Text(json.into())).await.is_err() {
    warn!("Client disconnected, stopping send loop");
    break;
}

// Or ignore errors for broadcast
let _ = sender.send(Message::Text(json.into())).await;
```

### Parse Errors

```rust
match serde_json::from_str::<ClientMessage>(&text) {
    Ok(msg) => {
        // Handle message
    }
    Err(e) => {
        warn!(error = %e, message = %text, "Invalid message format");
        // Send error response
        let error_msg = serde_json::json!({
            "type": "error",
            "message": "Invalid message format"
        });
        let _ = sender.send(Message::Text(
            serde_json::to_string(&error_msg).unwrap().into()
        )).await;
    }
}
```

## Connection Management

### Heartbeat / Keepalive

```rust
use tokio::time::{interval, Duration};

async fn handle_ws(socket: WebSocket, state: SharedState) {
    let (mut sender, mut receiver) = socket.split();
    let mut ping_interval = interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            _ = ping_interval.tick() => {
                // Send ping to keep connection alive
                if sender.send(Message::Ping(vec![])).await.is_err() {
                    break;
                }
            }

            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        // Handle message
                    }
                    Some(Ok(Message::Pong(_))) => {
                        // Client responded to ping
                        debug!("Received pong");
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
}
```

### Timeout on Inactivity

```rust
use tokio::time::{timeout, Duration};

async fn handle_ws(socket: WebSocket, state: SharedState) {
    let (mut sender, mut receiver) = socket.split();

    loop {
        // Wait for message with timeout
        let msg = timeout(Duration::from_secs(60), receiver.next()).await;

        match msg {
            Ok(Some(Ok(Message::Text(text)))) => {
                // Handle message
            }
            Ok(Some(Ok(Message::Close(_)))) | Ok(None) => {
                break;
            }
            Ok(_) => continue,
            Err(_) => {
                // Timeout - client inactive
                warn!("Client inactive, closing connection");
                let _ = sender.send(Message::Close(None)).await;
                break;
            }
        }
    }
}
```

### Graceful Shutdown

```rust
async fn handle_ws(socket: WebSocket, state: SharedState, shutdown: broadcast::Receiver<()>) {
    let (mut sender, mut receiver) = socket.split();
    let mut shutdown_rx = shutdown;

    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => {
                info!("Shutdown signal received");
                let _ = sender.send(Message::Close(None)).await;
                break;
            }

            msg = receiver.next() => {
                // Handle messages
            }
        }
    }
}
```

## Testing

### Testing WebSocket Handlers

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::ws::Message;
    use futures_util::{SinkExt, StreamExt};

    #[tokio::test]
    async fn test_websocket_echo() {
        let state = Arc::new(AppState::new());
        let app = Router::new()
            .route("/ws", get(ws_handler))
            .with_state(state);

        // Create test server
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        // Connect as client
        let (ws_stream, _) = tokio_tungstenite::connect_async(
            format!("ws://{}/ws", addr)
        ).await.unwrap();

        let (mut write, mut read) = ws_stream.split();

        // Send message
        write.send(Message::Text("hello".into())).await.unwrap();

        // Receive response
        let msg = read.next().await.unwrap().unwrap();
        assert_eq!(msg, Message::Text("hello".into()));
    }
}
```

## Common Patterns

### Discovery Service Pattern

**Simple broadcast of state updates**:
```rust
// State change triggers broadcast
async fn register_rover(state: &AppState, rover: RoverInfo) {
    let mut rovers = state.rovers.write().await;
    rovers.insert(rover.id.clone(), rover.clone());
    drop(rovers);

    // Notify all WebSocket clients
    state.notify();  // Sends empty signal on broadcast channel
}

// WebSocket handler receives signal and sends full state
async fn handle_ws(socket: WebSocket, state: SharedState) {
    let mut rx = state.broadcast_tx.subscribe();

    loop {
        tokio::select! {
            Ok(()) = rx.recv() => {
                let rovers = state.get_rovers().await;
                let msg = WsMessage {
                    msg_type: "rovers".to_string(),
                    data: rovers,
                };
                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = sender.send(Message::Text(json.into())).await;
                }
            }
            // ...
        }
    }
}
```

### Dispatch Service Pattern

**Bidirectional communication with different client types**:
```rust
// Two WebSocket endpoints
.route("/ws", get(ws_rover_handler))       // Rovers connect here
.route("/ws/console", get(ws_console_handler))  // Console connects here

// Rover WebSocket - bidirectional
async fn ws_rover_handler(ws: WebSocketUpgrade, state: SharedState) {
    ws.on_upgrade(|socket| handle_rover_ws(socket, state))
}

async fn handle_rover_ws(socket: WebSocket, state: SharedState) {
    let (tx, mut rx) = tokio::sync::mpsc::channel(32);
    let mut rover_id: Option<String> = None;

    // Register rover and store sender
    // ... registration logic ...

    // Bidirectional communication
    loop {
        tokio::select! {
            // Receive from rover
            msg = receiver.next() => {
                // Handle rover messages (register, progress, complete)
            }

            // Send to rover
            Some(msg) = rx.recv() => {
                // Forward messages from dispatch to rover
            }
        }
    }

    // Cleanup
    if let Some(id) = rover_id {
        state.rovers.write().await.remove(&id);
    }
}

// Console WebSocket - broadcast only
async fn ws_console_handler(ws: WebSocketUpgrade, state: SharedState) {
    ws.on_upgrade(|socket| handle_console_ws(socket, state))
}

async fn handle_console_ws(socket: WebSocket, state: SharedState) {
    let mut rx = state.broadcast_tx.subscribe();

    // Send initial state
    // ... send current rovers, tasks, etc. ...

    // Receive broadcast updates
    loop {
        tokio::select! {
            Ok(msg) = rx.recv() => {
                // Forward updates to console
            }
            msg = receiver.next() => {
                // Console messages (if any)
            }
        }
    }
}
```

## Performance Considerations

### Message Batching

```rust
use tokio::time::{interval, Duration};

async fn handle_ws(socket: WebSocket, state: SharedState) {
    let mut rx = state.broadcast_tx.subscribe();
    let mut batch: Vec<BroadcastMessage> = Vec::new();
    let mut flush_interval = interval(Duration::from_millis(100));

    loop {
        tokio::select! {
            // Collect messages
            Ok(msg) = rx.recv() => {
                batch.push(msg);
            }

            // Flush batch periodically
            _ = flush_interval.tick() => {
                if !batch.is_empty() {
                    // Send all messages at once
                    if let Ok(json) = serde_json::to_string(&batch) {
                        let _ = sender.send(Message::Text(json.into())).await;
                    }
                    batch.clear();
                }
            }
        }
    }
}
```

### Rate Limiting

```rust
use tokio::time::{interval, Duration};

async fn handle_ws(socket: WebSocket, state: SharedState) {
    let mut last_update = Instant::now();
    let min_interval = Duration::from_millis(50);  // Max 20 updates/sec

    loop {
        tokio::select! {
            Ok(msg) = rx.recv() => {
                let now = Instant::now();
                if now.duration_since(last_update) >= min_interval {
                    // Send update
                    if let Ok(json) = serde_json::to_string(&msg) {
                        let _ = sender.send(Message::Text(json.into())).await;
                    }
                    last_update = now;
                }
            }
        }
    }
}
```
