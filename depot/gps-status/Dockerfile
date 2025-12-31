# Build stage
FROM rust:1.83-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock* ./

# Create dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true

# Copy actual source and force rebuild
COPY src ./src
RUN touch src/main.rs

# Build for release
RUN cargo build --release

# Runtime stage
FROM alpine:3.21

RUN apk add --no-cache ca-certificates

COPY --from=builder /app/target/release/gps-status /usr/local/bin/gps-status

ENV PORT=4880
ENV GPS_SERIAL_PORT=/dev/ttyUSB0
ENV GPS_BAUD_RATE=115200

EXPOSE 4880

CMD ["gps-status"]
