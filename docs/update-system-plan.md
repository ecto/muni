# Muni Update System Plan

## Overview

Two-sided update system:
- **Developers**: `muni` CLI for deploying local changes
- **Customers**: Console UI for applying released updates

Release flow: GitHub Releases → Depot checks (manual or opt-in auto) → Customer applies via UI

**Auto-update is OFF by default** - customers must explicitly enable it.

---

## Architecture

```
GitHub Releases (you publish)
        │
        │ periodic check
        ▼
┌─────────────────────────────────────────────────────────────┐
│  Depot                                                       │
│                                                              │
│  ┌──────────┐    ┌─────────────┐    ┌──────────────────┐   │
│  │ updater  │───▶│ docker.sock │    │ Console UI       │   │
│  │ service  │    └─────────────┘    │ /system-updates  │   │
│  └────┬─────┘                       └────────┬─────────┘   │
│       │                                      │              │
│       │ REST API                             │ fetch        │
│       ▼                                      ▼              │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  /api/updater/                                       │   │
│  │    GET  /status     - versions, available updates    │   │
│  │    POST /check      - force check GitHub Releases    │   │
│  │    POST /apply      - apply depot update             │   │
│  │    POST /rover/:id  - push firmware to rover         │   │
│  │    POST /fleet      - push firmware to all rovers    │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                              │
└──────────────────────────────────────────────────────────────┘
        │
        │ SSH + SCP (like deploy.sh)
        ▼
    Rovers (firmware update)
```

---

## Components to Build

### 1. Updater Service (Rust)

New service: `depot/updater/`

**Responsibilities:**
- Poll GitHub Releases API (configurable interval, default 1hr)
- Store current/available versions
- Expose REST API for console
- Execute depot self-update (pull images, restart containers)
- Execute rover firmware updates (download binary, SSH push)

**Key files:**
```
depot/updater/
├── Cargo.toml
├── Dockerfile
└── src/
    ├── main.rs        # Axum server, routes
    ├── github.rs      # GitHub Releases API client
    ├── docker.rs      # Docker socket operations (bollard crate)
    ├── rover.rs       # Rover firmware push (SSH/SCP)
    └── state.rs       # Version tracking, update status
```

**Docker Compose addition:**
```yaml
updater:
  build: ./updater
  container_name: depot-updater
  restart: unless-stopped
  ports:
    - "4895:4895"  # Internal only, proxied via console
  volumes:
    - /var/run/docker.sock:/var/run/docker.sock:rw
    - updater-cache:/data/cache  # Downloaded firmware binaries
  environment:
    - PORT=4895
    - GITHUB_REPO=your-org/muni  # your repo
    - CHECK_INTERVAL=86400  # 24 hours (manual check always available)
    - AUTO_CHECK=false  # opt-in, disabled by default
    - AUTO_APPLY=false  # never auto-apply, always require user action
    - RUST_LOG=updater=info
```

### 2. Console System Updates Page

New view: `depot/console/src/views/SystemUpdatesView.tsx`

**UI sections:**
1. **Header** - "System Updates" + last check time + "Check Now" button
2. **Depot Services** - Current version, available version, "Update" button
3. **Fleet Firmware** - Per-rover version, online status, "Update" / "Update All" buttons
4. **Update Progress** - Real-time progress when update is running
5. **Settings** - Auto-check toggle (off by default), check interval

**Files to modify:**
- `src/App.tsx` - Add route `/system-updates`
- `src/components/AppSidebar.tsx` - Add menu item
- `src/lib/types.ts` - Add UpdateStatus types

### 3. CLI Commands

Extend `bvr/firmware/bins/cli/` (rename to `muni` binary):

```
muni deploy depot         # rsync local code, rebuild containers
muni deploy rover <id>    # cross-compile, push to rover
muni deploy --all         # depot + all rovers

muni update --check       # check GitHub for new releases
muni update --apply       # (for scripting, calls updater API)
```

**Note:** CLI deploy commands are for developers pushing local changes. The updater service handles customer updates from releases.

---

## Implementation Order

### Phase 1: Updater Service Foundation
1. Create `depot/updater/` Rust service scaffold
2. Implement GitHub Releases API client
3. Implement version comparison logic
4. Add to docker-compose.yml
5. Test: service starts, checks releases, exposes `/status`

### Phase 2: Depot Self-Update
1. Implement Docker socket operations (bollard crate)
2. Add `/apply` endpoint - pulls images, restarts services
3. Handle self-restart gracefully (updater restarts last)
4. Test: trigger update, verify services restart with new images

### Phase 3: Console UI
1. Create `SystemUpdatesView.tsx` with basic layout
2. Add to routing and sidebar
3. Fetch from `/api/updater/status`
4. Add "Check Now" and "Update Depot" buttons
5. Add progress indicator during updates

### Phase 4: Rover Firmware Updates
1. Add rover firmware download (from GitHub Release assets)
2. Implement SSH/SCP push logic (port deploy.sh to Rust)
3. Add `/rover/:id` and `/fleet` endpoints
4. Update console UI with rover update section
5. Test: push firmware to rover, verify restart

### Phase 5: CLI Deploy Commands
1. Add `muni deploy` subcommand structure
2. Implement `deploy depot` (rsync + docker compose)
3. Implement `deploy rover` (cross-compile + push)
4. Test: developer workflow end-to-end

---

## API Specification

### GET /api/updater/status
```json
{
  "depot": {
    "current": "1.2.0",
    "available": "1.3.0",
    "status": "update_available"  // up_to_date | update_available | updating | failed
  },
  "rovers": [
    {
      "id": "frog-0",
      "name": "Frog Zero",
      "online": true,
      "current": "0.8.1",
      "available": "0.9.0",
      "status": "update_available"
    }
  ],
  "lastCheck": "2025-01-20T15:30:00Z",
  "autoCheck": false,  // opt-in, off by default
  "checkInterval": 86400
}
```

### POST /api/updater/check
Force check for updates. Returns same as `/status`.

### POST /api/updater/apply
Start depot update. Returns:
```json
{
  "status": "started",
  "steps": ["pulling images", "restarting services"]
}
```

### POST /api/updater/rover/:id
Start rover firmware update. Returns:
```json
{
  "status": "started",
  "roverId": "frog-0"
}
```

### GET /api/updater/progress (WebSocket)
Real-time update progress:
```json
{
  "type": "depot_update",
  "step": "restarting console",
  "progress": 75,
  "message": "Restarting console service..."
}
```

---

## Key Files to Create/Modify

**New files:**
- `depot/updater/` (entire new service)
- `depot/console/src/views/SystemUpdatesView.tsx`
- `depot/console/src/hooks/useUpdater.ts`

**Modified files:**
- `depot/docker-compose.yml` - add updater service
- `depot/console/src/App.tsx` - add route
- `depot/console/src/components/AppSidebar.tsx` - add menu item
- `depot/console/src/lib/types.ts` - add types
- `depot/console/nginx.conf` - add `/api/updater/` proxy
- `bvr/firmware/bins/cli/src/main.rs` - add deploy subcommand

---

## Verification

1. **Updater service health:**
   ```bash
   curl http://localhost:4895/health
   curl http://localhost:4895/status
   ```

2. **Console UI:**
   - Navigate to /system-updates
   - Verify versions display correctly
   - Click "Check Now", verify API call
   - Click "Update", verify progress shows

3. **End-to-end depot update:**
   - Publish test release to GitHub
   - Wait for auto-check (or click Check Now)
   - Click Update, verify services restart
   - Verify new version shows after restart

4. **Rover update:**
   - With rover online, click Update on rover row
   - Verify firmware pushed via SSH
   - Verify rover restarts with new version

5. **CLI deploy:**
   ```bash
   muni deploy depot        # from dev machine
   muni deploy rover frog-0
   ```
