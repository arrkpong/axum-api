# 🏗️ AXUM API

<div align="center">

[![CI](https://img.shields.io/github/actions/workflow/status/arrkpong/axum-api/docker-image.yml?style=for-the-badge&label=Build)](https://github.com/arrkpong/axum-api/actions)
[![Rust](https://img.shields.io/badge/rust-1.92%2B-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org)
[![Docker](https://img.shields.io/badge/docker-ready-0db7ed?style=for-the-badge&logo=docker&logoColor=white)](https://www.docker.com/)
[![License](https://img.shields.io/badge/license-MIT-green?style=for-the-badge)](LICENSE)

**High-performance REST API** for the Axum ecosystem.
Built with **Rust (Axum)** • Shielded by **Pingora** • Powered by **PostgreSQL**

[Getting Started](#-getting-started) •
[API Reference](#-api-reference) •
[Documentation](#-documentation) •
[Contributing](#-contributing)

</div>

---

## 📖 Overview

Axum API is a production-ready backend service designed for high throughput and security. It provides authentication, user management, and serves as the foundation for the Axum platform.

### ✨ Key Features

- **🚀 High Performance** — Async Rust runtime with a lightweight Pingora proxy
- **🛡️ Secure by Default** — Argon2 hashing, JWT rotation, rate limiting
- **🦀 Modern Rust Stack** — Axum + Pingora + SQLx + Tokio
- **🐳 Cloud Native** — Multi-stage Docker builds on Ubuntu 22.04 LTS
- **📚 Documented Stack** — Local crate notes and API reference

---

## 🏗️ Architecture

```mermaid
flowchart TB
    subgraph Internet["🌐 Internet"]
        Client["👤 Client"]
    end

    subgraph Edge["🛡️ Edge Layer"]
        Proxy["Pingora Proxy<br/>(L1 HTTP Cache)"]
    end

    subgraph Application["⚙️ Application Layer"]
        API["Axum API"]
    end

    subgraph Data["💾 Data Layer"]
        Cache["Dragonfly<br/>(L2 Data Cache)"]
        DB[("PostgreSQL")]
    end

    Client -->|"Port 80"| Proxy
    Proxy -->|"Port 8080"| API
    API <-->|"Port 6379"| Cache
    API -->|"Port 5432"| DB

    %% High Contrast Styling
    style Internet fill:#E3F2FD,stroke:#1565C0,stroke-width:2px
    style Edge fill:#FCE4EC,stroke:#C2185B,stroke-width:2px
    style Application fill:#EDE7F6,stroke:#512DA8,stroke-width:2px
    style Data fill:#E8F5E9,stroke:#2E7D32,stroke-width:2px

    style Client fill:#1565C0,stroke:#0D47A1,stroke-width:2px,color:#FFF
    style Proxy fill:#C2185B,stroke:#880E4F,stroke-width:2px,color:#FFF
    style API fill:#512DA8,stroke:#311B92,stroke-width:2px,color:#FFF
    style Cache fill:#FF6F00,stroke:#E65100,stroke-width:2px,color:#FFF
    style DB fill:#2E7D32,stroke:#1B5E20,stroke-width:2px,color:#FFF
```

| Service           | Port | Description                                |
| ----------------- | ---- | ------------------------------------------ |
| **Pingora Proxy** | 80   | Edge gateway, L1 HTTP cache, rate limiting |
| **Axum API**      | 8080 | Business logic, authentication             |
| **Dragonfly**     | 6379 | L2 data cache (sessions, query results)    |
| **PostgreSQL**    | 5432 | Primary data store                         |

---

## 🛠️ Tech Stack

| Category      | Technology                                              | Purpose                         |
| ------------- | ------------------------------------------------------- | ------------------------------- |
| **Framework** | [Axum 0.8](https://github.com/tokio-rs/axum)            | Ergonomic web framework         |
| **Proxy**     | [Pingora 0.6](https://github.com/cloudflare/pingora)    | High-performance reverse proxy  |
| **Database**  | [SQLx 0.8](https://github.com/launchbadge/sqlx)         | Async, compile-time checked SQL |
| **Runtime**   | [Tokio 1.0](https://tokio.rs)                           | Async runtime                   |
| **Auth**      | [jsonwebtoken](https://github.com/Keats/jsonwebtoken)   | JWT access/refresh tokens       |
| **Hashing**   | [Argon2](https://github.com/RustCrypto/password-hashes) | Password hashing                |
| **Docs**      | Markdown docs                                           | Project and crate notes         |

---

## 📋 Prerequisites

Before you begin, ensure you have:

- **Docker Desktop** v24+ ([Download](https://www.docker.com/products/docker-desktop/))
- **Git** ([Download](https://git-scm.com/downloads))

_For local development (optional):_

- **Rust** v1.92+ ([Install](https://rustup.rs/))
- **PostgreSQL** v16+ ([Download](https://www.postgresql.org/download/))
- **SQLx CLI** (`cargo install sqlx-cli`)

---

## 🚀 Getting Started

### Quick Deploy (Docker)

```bash
# 1. Clone repository
git clone https://github.com/arrkpong/axum-api.git
cd axum-api

# 2. Configure environment
cp .env.example .env
# ⚠️ Edit .env and set secure values for JWT_SECRET and POSTGRES_PASSWORD

# 3. Start all services
docker compose up -d --build

# 4. Verify deployment
docker compose ps
curl http://localhost/
```

### Local Development

```bash
# 1. Start PostgreSQL (via Docker or local install)
docker compose up -d db

# 2. Configure environment
cp .env.example .env
# Set DATABASE_URL=postgres://postgres:password@localhost:5432/axum_db
# The repo enables SQLX_OFFLINE=true for local tooling via .cargo/config.toml.
# For local runs, also set REDIS_URL=redis://localhost:6379.
# The Docker Compose defaults use db / dragonfly instead.

# 3. Run migrations
sqlx database create
sqlx migrate run

# 4. Start development server
cargo run
```

---

## 🔧 Configuration

All configuration is done via environment variables. See `.env.example` for the complete list.

### Required Variables

| Variable            | Description                  | Example                             |
| ------------------- | ---------------------------- | ----------------------------------- |
| `DATABASE_URL`      | PostgreSQL connection string | `postgres://user:pass@host:5432/db` |
| `JWT_SECRET`        | Secret key for token signing | `<32+ character random string>`     |
| `POSTGRES_PASSWORD` | Database password            | `<strong random password>`          |

### Optional Variables

| Variable              | Default | Description                 |
| --------------------- | ------- | --------------------------- |
| `CORS_ORIGIN`         | `*`     | Allowed CORS origins        |
| `RATE_LIMIT_REQUESTS` | `300`   | Max requests per window     |
| `RATE_LIMIT_SECONDS`  | `60`    | Rate limit window (seconds) |
| `LOG_LEVEL`           | `info`  | Logging verbosity           |

### Generate Secure Secrets

```bash
# Linux/macOS
openssl rand -base64 32

# PowerShell
$bytes = New-Object byte[] 32
[Security.Cryptography.RandomNumberGenerator]::Create().GetBytes($bytes)
[Convert]::ToBase64String($bytes)
```

---

## 📚 API Reference

### Authentication Endpoints

| Method | Endpoint                | Description              | Auth |
| ------ | ----------------------- | ------------------------ | :--: |
| `POST` | `/api/v1/register` | Register new user        |  ❌  |
| `POST` | `/api/v1/login`    | Login & get tokens       |  ❌  |
| `POST` | `/api/v1/refresh`  | Refresh access token     |  ❌  |
| `POST` | `/api/v1/logout`   | Logout (blacklist token) |  ✅  |

### User Endpoints

| Method | Endpoint               | Description      | Auth |
| ------ | ---------------------- | ---------------- | :--: |
| `GET`  | `/api/v1/profile` | Get current user |  ✅  |

### System Endpoints

| Method | Endpoint      | Description          | Auth |
| ------ | ------------- | -------------------- | :--: |
| `GET`  | `/` | Welcome / health check |  ❌  |

---

## 📂 Project Structure

```
axum-api/
├── 📂 src/                    # API source code
│   ├── 📂 handlers/           # Request controllers
│   ├── 📂 models/             # Data structures & DTOs
│   ├── 📂 utils/              # Helpers (JWT, hashing)
│   ├── config.rs              # Configuration management
│   ├── state.rs               # Application state
│   └── main.rs                # Entry point
├── 📂 pingora_proxy/          # Reverse proxy service
│   ├── src/main.rs            # Proxy logic
│   └── Dockerfile
├── 📂 migrations/             # Database migrations
├── 📂 docs/                   # Additional documentation
├── 📜 Cargo.toml              # Rust dependencies
├── 📜 Dockerfile              # API container build
├── 📜 docker-compose.yml      # Service orchestration
├── 📜 .env.example            # Environment template
└── 📜 README.md               # This file
```

---

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

> Current test suites compile successfully, but there are no unit or integration test cases yet.

### Performance Testing

Use `oha` for load testing on this project. `ab` can time out on Docker Desktop / host networking and is less reliable here.

```bash
# Direct API
oha -n 5000 -c 50 http://localhost:8080/
oha -n 5000 -c 100 http://localhost:8080/
oha -n 5000 -c 200 http://localhost:8080/

# Through Pingora proxy
oha -n 5000 -c 50 http://localhost/
oha -n 5000 -c 100 http://localhost/
oha -n 5000 -c 200 http://localhost/

# Built-in benchmark routes
oha -n 5000 -c 50 http://localhost:8080/benchmark/io
oha -n 5000 -c 50 http://localhost:8080/benchmark/db-read
oha -n 200 -c 10 http://localhost:8080/benchmark/cpu
oha -n 200 -c 10 http://localhost:8080/benchmark/db-write
```

For the login benchmark on PowerShell, create a temporary body file first:

```powershell
$loginBody = Join-Path $env:TEMP 'oha-login-body.json'
Set-Content -Path $loginBody -Value '{"username":"benchuser","password":"Password123!"}' -NoNewline
oha -n 1000 -c 20 -m POST -H "Content-Type: application/json" -D $loginBody http://localhost:8080/api/v1/login
```

Benchmark-only routes are disabled by default. To enable `/benchmark/*`, set
`ENABLE_BENCHMARK_ROUTES=true` in `.env`, then rebuild the API container.

`/benchmark/db-write` inserts rows into `auth_users`. Run it sparingly or on a
throwaway database if you want to avoid growing the table during repeated tests.

If you want to remove middleware from the result while testing raw throughput, raise these values first in `.env`:

```env
RATE_LIMIT_REQUESTS=100000
REQUEST_TIMEOUT_SECONDS=60
```

### Measured Results

Local benchmark runs were taken with `oha` against `GET /` using the Docker stack on this machine.
The direct API measurements below were captured at `c=100` and `c=200`; the proxy measurements were captured at `c=50`, `c=100`, `c=200`, and `c=300`.

| Target | Concurrency | Requests/sec | Avg latency | p95 | p99 |
| ------ | ----------- | ------------ | ----------- | --- | --- |
| `http://localhost:8080/` | 100 | 38,922.9 | 2.44 ms | 4.10 ms | 9.18 ms |
| `http://localhost:8080/` | 200 | 39,905.5 | 4.73 ms | 7.82 ms | 44.66 ms |
| `http://localhost/`      | 50  | 24,849.2 | 1.97 ms | 2.79 ms | 4.78 ms |
| `http://localhost/`      | 100 | 21,980.9 | 4.42 ms | 6.13 ms | 16.19 ms |
| `http://localhost/`      | 200 | 19,756.8 | 9.67 ms | 13.62 ms | 40.80 ms |
| `http://localhost/`      | 300 | 9,569.3  | 18.55 ms | 17.41 ms | 510.04 ms |

These numbers are workload-specific and will change with CPU, Docker networking, rate limit settings, and whether the proxy is in the path.

### Native Local Results

These runs were taken against the app started with `cargo run` on the host machine,
using the local Postgres instance on `localhost:5432` after applying the auth and
token migrations.

| Target | Concurrency | Requests/sec | Avg latency | p95 | p99 |
| ------ | ----------- | ------------ | ----------- | --- | --- |
| `http://127.0.0.1:8082/` | 50  | 2,122.2 | 23.50 ms | 57.85 ms | 67.36 ms |
| `http://127.0.0.1:8082/` | 100 | 2,572.1 | 38.49 ms | 62.46 ms | 73.21 ms |
| `http://127.0.0.1:8082/` | 200 | 3,482.4 | 56.14 ms | 68.04 ms | 564.96 ms |
| `http://127.0.0.1:8082/api/v1/login` | 20 | 45.4 | 440.72 ms | 486.36 ms | 570.68 ms |
| `http://127.0.0.1:8082/benchmark/io` | 50 | 860.4 | 58.04 ms | 64.19 ms | 68.24 ms |
| `http://127.0.0.1:8082/benchmark/db-read` | 50 | 4,222.3 | 11.76 ms | 17.74 ms | 20.33 ms |
| `http://127.0.0.1:8082/benchmark/cpu` | 10 | 846.8 | 11.48 ms | 16.51 ms | 21.01 ms |
| `http://127.0.0.1:8082/benchmark/db-write` | 10 | 23.4 | 413.72 ms | 450.73 ms | 772.64 ms |

If you want to benchmark auth routes locally, make sure your local Postgres has
the migrations applied. The Docker `db` service is not exposed on `localhost:5432`
unless you add a port mapping yourself.

### Application Route Results

The table below captures the direct API runs for the routes that exercise auth,
CPU, I/O, and database paths. The login benchmark used a pre-created `benchuser`
account so the request path exercised token issuance instead of validation errors.

| Target | Concurrency | Requests/sec | Avg latency | p95 | p99 |
| ------ | ----------- | ------------ | ----------- | --- | --- |
| `http://localhost:8080/api/v1/login` | 20 | 207.4 | 96.00 ms | 112.38 ms | 121.32 ms |
| `http://localhost:8080/benchmark/io` | 50 | 946.4 | 52.74 ms | 53.91 ms | 54.80 ms |
| `http://localhost:8080/benchmark/db-read` | 50 | 13,715.2 | 3.60 ms | 4.09 ms | 4.90 ms |
| `http://localhost:8080/benchmark/cpu` | 10 | 4,007.2 | 2.41 ms | 4.48 ms | 6.37 ms |
| `http://localhost:8080/benchmark/db-write` | 10 | 201.0 | 49.07 ms | 62.64 ms | 73.39 ms |

---

## 🐳 Deployment

### Production Checklist

- [ ] Set strong `JWT_SECRET` (32+ characters)
- [ ] Set strong `POSTGRES_PASSWORD`
- [ ] Configure `CORS_ORIGIN` to your domain
- [ ] Set appropriate `RATE_LIMIT_*` values
- [ ] Enable HTTPS (via reverse proxy or load balancer)
- [ ] Set up database backups

### Docker Compose Commands

```bash
# Start all services
docker compose up -d --build

# View logs
docker compose logs -f

# Stop all services
docker compose down

# Rebuild specific service
docker compose up -d --build api
```

---

## 🤝 Contributing

Contributions are welcome! Please follow these steps:

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Commit** your changes (`git commit -m 'Add amazing feature'`)
4. **Push** to the branch (`git push origin feature/amazing-feature`)
5. **Open** a Pull Request

### Development Guidelines

- Follow Rust formatting (`cargo fmt`)
- Ensure all tests pass (`cargo test`)
- Add tests for new features
- Update documentation as needed

---

## 📄 License

This project is licensed under the **MIT License** — see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgements

- [Axum](https://github.com/tokio-rs/axum) — Ergonomic web framework
- [Pingora](https://github.com/cloudflare/pingora) — Cloudflare's proxy framework
- [SQLx](https://github.com/launchbadge/sqlx) — Async SQL toolkit

---

<div align="center">

**Built with ❤️ and 🦀 by the Axum Team**

</div>
