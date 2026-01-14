# Docker Deployment Patterns

Comprehensive guide for Docker containerization and deployment of depot services.

## Overview

All depot services use:
- **Multi-stage builds** for minimal image sizes (10-20 MB)
- **Alpine Linux** as base image
- **Docker Compose** for orchestration
- **Health checks** for reliability

## Dockerfile Pattern

### Multi-Stage Build

```dockerfile
# =============================================================================
# Build Stage
# =============================================================================
FROM rust:1.83-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev

WORKDIR /app

# Copy dependency manifests
COPY Cargo.toml Cargo.lock* ./

# Create dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (cached layer)
RUN cargo build --release 2>/dev/null || true

# Copy actual source code
COPY src ./src

# Force rebuild by touching source
RUN touch src/main.rs

# Build for release
RUN cargo build --release

# =============================================================================
# Runtime Stage
# =============================================================================
FROM alpine:3.21

# Install runtime dependencies
RUN apk add --no-cache ca-certificates

# Copy binary from builder
COPY --from=builder /app/target/release/discovery /usr/local/bin/discovery

# Set environment variables
ENV PORT=4860
ENV RUST_LOG=info

# Expose port
EXPOSE 4860

# Run binary
CMD ["discovery"]
```

### Why Multi-Stage Builds?

**Without multi-stage**:
- Image size: ~1.5 GB (includes Rust compiler, build tools, etc.)
- Attack surface: large (many unnecessary packages)
- Slow to transfer

**With multi-stage**:
- Image size: ~15 MB (only binary + Alpine + ca-certificates)
- Attack surface: minimal (only runtime dependencies)
- Fast to transfer and deploy

### Build Stage Optimization

**Dependency Caching**:
```dockerfile
# Copy only Cargo files first
COPY Cargo.toml Cargo.lock* ./

# Create dummy source to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true

# Now copy real source (dependencies are cached)
COPY src ./src
RUN touch src/main.rs  # Force rebuild
RUN cargo build --release
```

**Why this works**:
- Docker layers are cached based on file changes
- If `Cargo.toml` doesn't change, dependency layer is reused
- Only source code changes trigger recompilation
- Saves minutes on each build

### Runtime Stage Configuration

**Minimal runtime dependencies**:
```dockerfile
FROM alpine:3.21

# Only install what's needed at runtime
RUN apk add --no-cache \
    ca-certificates  # For HTTPS requests
```

**Common runtime dependencies**:
- `ca-certificates` - SSL/TLS certificates for HTTPS
- `libgcc` - C runtime (if using certain Rust features)
- `libssl` - OpenSSL (if not using rustls)

### Release Profile Optimization

In `Cargo.toml`:
```toml
[profile.release]
lto = true           # Link-time optimization
opt-level = "z"      # Optimize for size
strip = true         # Remove debug symbols
codegen-units = 1    # Better optimization (slower compile)
```

**Impact**:
- Binary size: ~8 MB → ~2 MB
- Runtime performance: same or better
- Compile time: +20-30%

## Docker Compose

### Basic Service Definition

```yaml
services:
  discovery:
    build:
      context: ./discovery
      dockerfile: Dockerfile
    container_name: depot-discovery
    restart: unless-stopped
    ports:
      - "4860:4860"
    environment:
      - PORT=4860
      - RUST_LOG=discovery=info
    healthcheck:
      test: ["CMD", "wget", "-q", "-O-", "http://localhost:4860/health"]
      interval: 10s
      timeout: 3s
      retries: 3
```

### Environment Variables

**From .env file**:
```yaml
services:
  dispatch:
    environment:
      - DATABASE_URL=${DATABASE_URL}
      - PORT=${DISPATCH_PORT:-4890}
      - RUST_LOG=${RUST_LOG:-dispatch=info,sqlx=warn}
```

**.env file**:
```bash
DATABASE_URL=postgres://postgres:password@postgres:5432/dispatch
DISPATCH_PORT=4890
RUST_LOG=dispatch=debug
```

**Variable substitution**:
- `${VAR}` - required, fails if not set
- `${VAR:-default}` - optional with default value
- `${VAR-default}` - optional, empty string if unset

### Service Dependencies

```yaml
services:
  console:
    depends_on:
      - discovery
      - dispatch
      - grafana
    # ...

  dispatch:
    depends_on:
      postgres:
        condition: service_healthy  # Wait for health check
    # ...

  postgres:
    healthcheck:
      test: ["CMD", "pg_isready", "-U", "postgres"]
      interval: 5s
      timeout: 3s
      retries: 5
```

**Dependency types**:
- **No condition** (default): Service starts after dependency starts
- **condition: service_healthy**: Service waits for dependency health check
- **condition: service_completed_successfully**: Service waits for dependency to exit with 0

### Volumes

**Named volumes** (persistent data):
```yaml
services:
  postgres:
    volumes:
      - postgres-data:/var/lib/postgresql/data
    # ...

volumes:
  postgres-data:  # Persisted across container restarts
```

**Bind mounts** (host directory):
```yaml
services:
  console:
    volumes:
      - ./console/dist:/usr/share/nginx/html:ro  # Read-only
      - /data/sessions:/data/sessions  # Read-write
```

**tmpfs** (in-memory, ephemeral):
```yaml
services:
  discovery:
    tmpfs:
      - /tmp
      - /var/run
```

### Networks

```yaml
services:
  discovery:
    networks:
      - depot-network
    # ...

  dispatch:
    networks:
      - depot-network
      - database-network
    # ...

networks:
  depot-network:
    driver: bridge
  database-network:
    driver: bridge
    internal: true  # No external access
```

### Restart Policies

```yaml
services:
  discovery:
    restart: unless-stopped  # Restart unless manually stopped
    # OR
    restart: always          # Always restart (even after manual stop)
    # OR
    restart: on-failure      # Only restart on crash
    # OR
    restart: "no"            # Never restart (default)
```

**Best practices**:
- Production services: `unless-stopped` or `always`
- Development: `unless-stopped` (allows manual stop)
- One-time jobs: `"no"`

### Health Checks

**HTTP health check**:
```yaml
healthcheck:
  test: ["CMD", "wget", "-q", "-O-", "http://localhost:4860/health"]
  interval: 10s      # Check every 10 seconds
  timeout: 3s        # Fail if takes longer than 3s
  retries: 3         # Mark unhealthy after 3 failures
  start_period: 30s  # Grace period on startup
```

**Database health check**:
```yaml
healthcheck:
  test: ["CMD", "pg_isready", "-U", "postgres"]
  interval: 5s
  timeout: 3s
  retries: 5
```

**Custom script health check**:
```yaml
healthcheck:
  test: ["CMD", "/usr/local/bin/health-check.sh"]
  interval: 30s
  timeout: 10s
  retries: 3
```

**Why health checks?**
- Docker knows when service is ready
- `depends_on` with `service_healthy` waits for health
- Orchestrators (Swarm, Kubernetes) can restart unhealthy containers

### Profiles

**Development vs Production**:
```yaml
services:
  # Always runs
  discovery:
    # ...

  # Only in GPU profile
  splat-worker:
    profiles:
      - gpu
    # ...

  # Only in RTK profile
  rtk-base:
    profiles:
      - rtk
    # ...
```

**Usage**:
```bash
# Run without profiles (base services only)
docker compose up -d

# Run with GPU profile
docker compose --profile gpu up -d

# Run with multiple profiles
docker compose --profile gpu --profile rtk up -d
```

## Common Docker Compose Commands

### Build and Start

```bash
# Build all services
docker compose build

# Build specific service
docker compose build discovery

# Build without cache
docker compose build --no-cache

# Start all services in background
docker compose up -d

# Start specific services
docker compose up -d discovery dispatch

# Build and start
docker compose up -d --build
```

### View Logs

```bash
# All services
docker compose logs

# Follow logs (live)
docker compose logs -f

# Specific service
docker compose logs -f discovery

# Last N lines
docker compose logs --tail=100 discovery

# Since timestamp
docker compose logs --since="2024-01-14T10:00:00"
```

### Restart Services

```bash
# Restart all
docker compose restart

# Restart specific service
docker compose restart discovery

# Stop and start (recreate containers)
docker compose down && docker compose up -d
```

### Stop and Remove

```bash
# Stop services (keep containers)
docker compose stop

# Stop and remove containers
docker compose down

# Remove containers, volumes, and networks
docker compose down -v

# Remove everything including images
docker compose down --rmi all -v
```

### Execute Commands

```bash
# Execute command in running container
docker compose exec discovery sh

# Execute as root
docker compose exec -u root discovery sh

# One-off command
docker compose run --rm discovery cargo --version
```

### View Status

```bash
# List running services
docker compose ps

# View service details
docker compose ps discovery

# View resource usage
docker stats
```

## Docker Registry

### Build and Tag

```bash
# Build with tag
docker build -t depot-discovery:v1.0.0 ./discovery

# Tag existing image
docker tag depot-discovery:v1.0.0 registry.example.com/depot-discovery:v1.0.0

# Multiple tags
docker tag depot-discovery:v1.0.0 depot-discovery:latest
```

### Push to Registry

```bash
# Log in to registry
docker login registry.example.com

# Push image
docker push registry.example.com/depot-discovery:v1.0.0

# Push all tags
docker push --all-tags registry.example.com/depot-discovery
```

### Pull from Registry

```bash
# Pull image
docker pull registry.example.com/depot-discovery:v1.0.0

# Pull and run
docker run -p 4860:4860 registry.example.com/depot-discovery:v1.0.0
```

## Debugging

### Container Not Starting

```bash
# View logs
docker compose logs discovery

# Check exit code
docker compose ps discovery

# Run with shell override
docker compose run --rm discovery sh

# Check healthcheck
docker inspect depot-discovery | jq '.[0].State.Health'
```

### Network Issues

```bash
# List networks
docker network ls

# Inspect network
docker network inspect depot_default

# Test connectivity
docker compose exec discovery ping dispatch
docker compose exec discovery wget -O- http://dispatch:4890/health
```

### Volume Issues

```bash
# List volumes
docker volume ls

# Inspect volume
docker volume inspect depot_postgres-data

# Check mount points
docker compose exec postgres df -h

# Backup volume
docker run --rm -v depot_postgres-data:/data -v $(pwd):/backup alpine tar czf /backup/postgres-backup.tar.gz /data
```

### Performance Issues

```bash
# View resource usage
docker stats

# Check container limits
docker inspect depot-discovery | jq '.[0].HostConfig.Memory'

# Set resource limits
docker compose:
  services:
    discovery:
      mem_limit: 512m
      cpus: 0.5
```

## Production Best Practices

### 1. Use Specific Tags

❌ BAD:
```dockerfile
FROM rust:latest  # Unpredictable
FROM alpine:latest
```

✅ GOOD:
```dockerfile
FROM rust:1.83-alpine  # Specific version
FROM alpine:3.21
```

### 2. Non-Root User

```dockerfile
# Create non-root user
RUN addgroup -g 1000 appuser && \
    adduser -D -u 1000 -G appuser appuser

# Copy binary
COPY --from=builder --chown=appuser:appuser /app/target/release/discovery /usr/local/bin/

# Switch to non-root
USER appuser

CMD ["discovery"]
```

### 3. Minimal Attack Surface

```dockerfile
# Only install what's needed
RUN apk add --no-cache ca-certificates

# Don't include dev tools in runtime
# ❌ apk add curl vim bash
```

### 4. Health Checks

```yaml
healthcheck:
  test: ["CMD", "wget", "-q", "-O-", "http://localhost:4860/health"]
  interval: 10s
  timeout: 3s
  retries: 3
  start_period: 30s
```

### 5. Resource Limits

```yaml
services:
  discovery:
    mem_limit: 256m
    cpus: 0.5
    deploy:
      resources:
        limits:
          memory: 256M
          cpus: '0.5'
        reservations:
          memory: 128M
          cpus: '0.25'
```

### 6. Secrets Management

```yaml
# Use Docker secrets (Swarm/Kubernetes)
services:
  dispatch:
    secrets:
      - database_password

secrets:
  database_password:
    file: ./secrets/database_password.txt

# Or use .env with restricted permissions
# chmod 600 .env
```

### 7. Logging

```yaml
services:
  discovery:
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

### 8. Monitoring

```yaml
services:
  discovery:
    labels:
      - "prometheus.io/scrape=true"
      - "prometheus.io/port=4860"
      - "prometheus.io/path=/metrics"
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Build and Push

on:
  push:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Login to Registry
        uses: docker/login-action@v3
        with:
          registry: registry.example.com
          username: ${{ secrets.REGISTRY_USERNAME }}
          password: ${{ secrets.REGISTRY_PASSWORD }}

      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: ./depot/discovery
          push: true
          tags: registry.example.com/depot-discovery:${{ github.sha }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

### GitLab CI

```yaml
build:
  stage: build
  image: docker:latest
  services:
    - docker:dind
  script:
    - docker login -u $CI_REGISTRY_USER -p $CI_REGISTRY_PASSWORD $CI_REGISTRY
    - docker build -t $CI_REGISTRY_IMAGE:$CI_COMMIT_SHA ./depot/discovery
    - docker push $CI_REGISTRY_IMAGE:$CI_COMMIT_SHA
```

## Troubleshooting Checklist

### Service Won't Start

- [ ] Check logs: `docker compose logs -f <service>`
- [ ] Check Dockerfile for errors
- [ ] Verify environment variables set
- [ ] Check port conflicts: `lsof -i :<port>`
- [ ] Verify health check endpoint works
- [ ] Check disk space: `df -h`
- [ ] Check memory: `free -h`

### Can't Connect Between Services

- [ ] Verify both services are in same network
- [ ] Use service name as hostname (not localhost)
- [ ] Check firewall rules
- [ ] Verify ports are exposed
- [ ] Check DNS resolution: `docker compose exec <service> nslookup <other-service>`

### Database Connection Failed

- [ ] Check DATABASE_URL is correct
- [ ] Verify database is running: `docker compose ps postgres`
- [ ] Check database health: `docker compose exec postgres pg_isready`
- [ ] Verify network connectivity
- [ ] Check database logs: `docker compose logs postgres`
- [ ] Verify user/password correct
- [ ] Check connection pool settings

### Build is Slow

- [ ] Use dependency caching pattern (dummy main.rs)
- [ ] Enable BuildKit: `DOCKER_BUILDKIT=1 docker build`
- [ ] Use `--cache-from` with registry cache
- [ ] Reduce image layers (combine RUN commands)
- [ ] Use multi-stage build
- [ ] Check internet connection (for downloads)

### Image is Too Large

- [ ] Use multi-stage build
- [ ] Use Alpine base image
- [ ] Remove build dependencies from runtime
- [ ] Optimize release profile in Cargo.toml
- [ ] Strip binary: `strip = true` in release profile
- [ ] Clean up in same layer: `RUN apk add ... && apk del ...`
