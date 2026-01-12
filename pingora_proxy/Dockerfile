# Build Stage
FROM rust:1.92.0-bookworm AS builder

WORKDIR /usr/src/app
COPY . .

# Install comprehensive build dependencies for Pingora (and its BoringSSL dependency)
# Pingora needs: CMake, Clang (for bindgen), Go (for BoringSSL), Perl, NASM
RUN apt-get update && apt-get install -y \
    cmake \
    clang \
    libclang-dev \
    libssl-dev \
    pkg-config \
    build-essential \
    perl \
    nasm \
    golang-go \
    && rm -rf /var/lib/apt/lists/*

# Build release binary
RUN cargo build --release

# Runtime Stage
FROM ubuntu:22.04

# Install OpenSSL/CA certificates (Pingora needs them for upstream TLS if used)
RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/pingora_proxy /usr/local/bin/pingora_proxy

# Expose HTTP port
EXPOSE 80

CMD ["pingora_proxy"]
