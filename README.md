# 🏗️ Civil Park API

<div align="center">

[![CI](https://img.shields.io/github/actions/workflow/status/civil-park-international/civil-park-api/ci.yml?style=for-the-badge&label=Build)](https://github.com/civil-park-international/civil-park-api/actions)
[![Rust](https://img.shields.io/badge/rust-1.92%2B-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org)
[![Docker](https://img.shields.io/badge/docker-ready-0db7ed?style=for-the-badge&logo=docker&logoColor=white)](https://www.docker.com/)
[![License](https://img.shields.io/badge/license-MIT-green?style=for-the-badge)](LICENSE)

**High-performance REST API** for the Civil Park ecosystem.  
Built with **Rust (Axum)** • Shielded by **Cloudflare Pingora** • Powered by **PostgreSQL**

[Getting Started](#-getting-started) •
[API Reference](#-api-reference) •
[Documentation](#-documentation) •
[Contributing](#-contributing)

</div>

---

## 📖 Overview

Civil Park API is a production-ready backend service designed for high throughput and security. It provides authentication, user management, and serves as the foundation for the Civil Park platform.

### ✨ Key Features

- **🚀 High Performance** — Async Rust achieving 100k+ req/s potential
- **🛡️ Secure by Default** — Argon2 hashing, JWT rotation, rate limiting
- **🦀 Modern Rust Stack** — Axum + Pingora + SQLx + Tokio
- **🐳 Cloud Native** — Multi-stage Docker builds on Ubuntu 22.04 LTS
- **📚 Self-Documenting** — Auto-generated OpenAPI/Swagger UI

---

## 🏗️ Architecture

```mermaid
flowchart TB
    subgraph Internet["🌐 Internet"]
        Client["👤 Client<br/><small>Browser / Mobile / API Consumer</small>"]
    end

    subgraph Edge["🛡️ Edge Layer"]
        Proxy["<b>Pingora Proxy</b><br/>━━━━━━━━━━━━<br/>• Load Balancing<br/>• Rate Limiting<br/>• Health Checks"]
    end

    subgraph Application["⚙️ Application Layer"]
        API["<b>Axum API</b><br/>━━━━━━━━━━━━<br/>• REST Endpoints<br/>• JWT Auth<br/>• Business Logic"]
    end

    subgraph Data["💾 Data Layer"]
        DB[("<b>PostgreSQL</b><br/>━━━━━━━━━━━━<br/>• Users<br/>• Auth Tokens<br/>• App Data")]
        Cache["<b>Redis</b><br/><small>(Future)</small>"]
    end

    Client -->|"HTTP/1.1<br/>Port 80"| Proxy
    Proxy -->|"HTTP<br/>Port 8080"| API
    API -->|"TCP<br/>Port 5432"| DB
    API -.->|"TCP<br/>Port 6379"| Cache

    %% Styling
    style Internet fill:#e1f5fe,stroke:#01579b,stroke-width:2px
    style Edge fill:#fce4ec,stroke:#880e4f,stroke-width:2px
    style Application fill:#e8eaf6,stroke:#1a237e,stroke-width:2px
    style Data fill:#e8f5e9,stroke:#1b5e20,stroke-width:2px

    style Client fill:#fff,stroke:#01579b,stroke-width:2px
    style Proxy fill:#f48fb1,stroke:#880e4f,stroke-width:2px,color:#000
    style API fill:#7986cb,stroke:#1a237e,stroke-width:2px,color:#fff
    style DB fill:#81c784,stroke:#1b5e20,stroke-width:2px,color:#000
    style Cache fill:#a5d6a7,stroke:#1b5e20,stroke-width:1px,stroke-dasharray: 5 5
```

| Service           | Port | Description                                 |
| ----------------- | ---- | ------------------------------------------- |
| **Pingora Proxy** | 80   | Edge gateway, load balancing, rate limiting |
| **Axum API**      | 8080 | Core business logic, authentication         |
| **PostgreSQL**    | 5432 | Primary data store                          |

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
| **Docs**      | [Utoipa](https://github.com/juhaku/utoipa)              | OpenAPI generation              |

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
git clone https://github.com/civil-park-international/civil-park-api.git
cd civil-park-api

# 2. Configure environment
cp .env.example .env
# ⚠️ Edit .env and set secure values for JWT_SECRET and POSTGRES_PASSWORD

# 3. Start all services
docker compose up -d --build

# 4. Verify deployment
docker compose ps
curl http://localhost/health
```

### Local Development

```bash
# 1. Start PostgreSQL (via Docker or local install)
docker compose up -d db

# 2. Configure environment
cp .env.example .env
# Set DATABASE_URL=postgres://postgres:password@localhost:5432/axum_db

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
| `RATE_LIMIT_REQUESTS` | `100`   | Max requests per window     |
| `RATE_LIMIT_SECONDS`  | `60`    | Rate limit window (seconds) |
| `LOG_LEVEL`           | `info`  | Logging verbosity           |

### Generate Secure Secrets

```bash
# Linux/macOS
openssl rand -base64 32

# PowerShell
[Convert]::ToBase64String([Security.Cryptography.RandomNumberGenerator]::GetBytes(32))
```

---

## 📚 API Reference

### Authentication Endpoints

| Method | Endpoint                | Description              | Auth |
| ------ | ----------------------- | ------------------------ | :--: |
| `POST` | `/api/v1/auth/register` | Register new user        |  ❌  |
| `POST` | `/api/v1/auth/login`    | Login & get tokens       |  ❌  |
| `POST` | `/api/v1/auth/refresh`  | Refresh access token     |  ❌  |
| `POST` | `/api/v1/auth/logout`   | Logout (blacklist token) |  ✅  |

### User Endpoints

| Method | Endpoint               | Description      | Auth |
| ------ | ---------------------- | ---------------- | :--: |
| `GET`  | `/api/v1/user/profile` | Get current user |  ✅  |

### System Endpoints

| Method | Endpoint      | Description          | Auth |
| ------ | ------------- | -------------------- | :--: |
| `GET`  | `/health`     | Health check         |  ❌  |
| `GET`  | `/swagger-ui` | Interactive API docs |  ❌  |

---

## 📂 Project Structure

```
civil-park-api/
├── 📂 src/                    # API source code
│   ├── 📂 handlers/           # Request controllers
│   ├── 📂 models/             # Data structures & DTOs
│   ├── 📂 services/           # Business logic
│   ├── 📂 repositories/       # Database access layer
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

### Performance Testing

```bash
# Using Apache Benchmark (via Docker)
docker run --rm --net=host httpd:alpine ab -n 1000 -c 50 http://localhost/

# Using oha (Rust-based)
docker run --rm --net=host ghcr.io/hatoo/oha -n 1000 -c 50 http://localhost/
```

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

**Built with ❤️ and 🦀 by the Civil Park Team**

</div>
