# Build stage
FROM rust:1.88-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app

# Copy shared types crate (map-api depends on ../types)
COPY types ./types

# Copy map-api crate
COPY map-api ./map-api

WORKDIR /app/map-api

# Cache dependencies with dummy build
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true

# Copy actual source and rebuild
COPY map-api/src ./src
RUN touch src/main.rs
RUN cargo build --release

# Runtime stage
FROM alpine:3.21

RUN apk add --no-cache ca-certificates

COPY --from=builder /app/map-api/target/release/map-api /usr/local/bin/map-api

ENV MAPS_DIR=/data/maps
ENV PORT=4870
ENV RUST_LOG=map_api=info

VOLUME ["/data/maps"]

EXPOSE 4870

CMD ["map-api"]
