# Build stage
FROM rust:1.83-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock* ./

# Create dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true

# Copy actual source
COPY src ./src

# Build for release
RUN cargo build --release

# Runtime stage
FROM alpine:3.21

RUN apk add --no-cache ca-certificates

COPY --from=builder /app/target/release/mapper /usr/local/bin/mapper

ENV SESSIONS_DIR=/data/sessions
ENV MAPS_DIR=/data/maps
ENV RUST_LOG=mapper=info

VOLUME ["/data/sessions", "/data/maps"]

CMD ["mapper"]
