# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

#### Dispatch System (Mission Planning & Task Execution)

A complete dispatch system for automated mission planning and task execution across the rover fleet.

**Depot Services**:
- Dispatch service (`depot/dispatch/`) - Rust Axum service on port 4890 for centralized mission management
- PostgreSQL database for persistent storage of zones, missions, and tasks
- WebSocket endpoints for real-time rover communication (`/ws`) and console updates (`/ws/console`)
- Zone CRUD API supporting waypoint routes and GPS polygons
- Mission CRUD API with schedule management and rover assignment
- Task lifecycle management (pending -> assigned -> active -> completed/failed)

**Console Frontend**:
- Dispatch view (`depot/console/src/views/DispatchView.tsx`) for zone/mission management
- Real-time task monitoring with WebSocket updates
- Zone editor for waypoint and polygon definition
- Mission scheduler with rover assignment
- Navigation link in sidebar

**Rover Firmware**:
- Dispatch client crate (`bvr/firmware/crates/dispatch/`) for WebSocket communication with depot
- Auto-reconnect with exponential backoff
- Task assignment handling with automatic mode transition to Autonomous
- Waypoint navigation using dispatched coordinates
- Progress reporting as waypoints are reached (percent, waypoint index, lap count)
- Task completion and failure reporting

**Configuration**:
- `[dispatch]` section in `bvr.toml` for endpoint configuration
- `--no-dispatch` and `--dispatch-endpoint` CLI flags for bvrd
- Docker Compose updates with PostgreSQL and dispatch service

**Data Flow**:
```
Console -> Dispatch Service -> Database -> WebSocket -> Rover
                                               |
Rover executes waypoints <- Task assigned <---+
       |
       +---> Progress/Complete/Failed ---> Dispatch Service ---> Console
```

**Key Files**:
- `depot/dispatch/src/main.rs` - Dispatch service implementation
- `depot/dispatch/migrations/001_initial.sql` - Database schema
- `depot/console/src/views/DispatchView.tsx` - Mission planning UI
- `depot/console/src/hooks/useDispatch.ts` - API client and WebSocket hook
- `bvr/firmware/crates/dispatch/src/lib.rs` - Rover dispatch client
- `bvr/firmware/bins/bvrd/src/main.rs` - Dispatch integration in daemon (lines 577-596, 945-1003, 1023-1150)
