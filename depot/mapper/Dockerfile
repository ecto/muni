# Build stage
FROM rust:1.88-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app

# Copy shared types crate (mapper depends on ../types)
COPY types ./types

# Copy mapper crate
COPY mapper ./mapper

WORKDIR /app/mapper

# Cache dependencies with dummy build
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true

# Copy actual source and rebuild
COPY mapper/src ./src
RUN cargo build --release

# Runtime stage
FROM alpine:3.21

RUN apk add --no-cache ca-certificates

COPY --from=builder /app/mapper/target/release/mapper /usr/local/bin/mapper

ENV SESSIONS_DIR=/data/sessions
ENV MAPS_DIR=/data/maps
ENV RUST_LOG=mapper=info

VOLUME ["/data/sessions", "/data/maps"]

CMD ["mapper"]
