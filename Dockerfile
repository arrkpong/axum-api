# Build stage: compile the Rust binary with all dependencies.
FROM rust:1.92.0-slim-bookworm AS builder

WORKDIR /app

# Install curl for utoipa-swagger-ui build (downloads Swagger UI assets).
RUN apt-get update && apt-get install -y --no-install-recommends curl \
    && rm -rf /var/lib/apt/lists/*

# Copy only the manifests first so dependency layers can be cached.
COPY Cargo.toml Cargo.lock ./

# Copy SQLx offline cache for compile-time query verification.
COPY .sqlx .sqlx

# Create a dummy main.rs to compile dependencies without full source.
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Enable SQLx offline mode for builds without database access.
ENV SQLX_OFFLINE=true

# Build dependencies only (this layer will be cached).
RUN cargo build --release --locked

# Remove the dummy source to avoid stale artifacts.
RUN rm -rf src

# Copy the full project source.
COPY . .

# Update the modification time of source files to force a rebuild.
# (Cargo skips rebuild if source mtime < artifact mtime).
RUN touch src/main.rs

# Build the production binary (uses cached dependencies).
RUN cargo build --release --locked

# Runtime stage: minimal OS with only the compiled binary.
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install only required runtime packages (TLS certs for HTTPS calls).
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage.
COPY --from=builder /app/target/release/axum-api /app/axum-api

# Create a non-root user for better security and take ownership.
RUN groupadd -r app && useradd -r -g app -d /app -s /usr/sbin/nologin app \
    && chown -R app:app /app

# Drop privileges to the non-root user.
USER app

# Default bind address and port; can be overridden at runtime.
ENV HOST=0.0.0.0
ENV PORT=8080

# Document the port the app listens on.
EXPOSE 8080

# Start the application.
CMD ["/app/axum-api"]
