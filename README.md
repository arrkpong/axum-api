# Axum API (Production Ready)

![Rust](https://img.shields.io/badge/Rust-1.92-orange?logo=rust)
![Axum](https://img.shields.io/badge/Axum-0.8.8-blue)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL-18.1-336791?logo=postgresql&logoColor=white)
![Docker](https://img.shields.io/badge/Docker-Ready-2496ED?logo=docker&logoColor=white)
![License](https://img.shields.io/badge/License-MIT-green)

A robust, production-ready REST API built with [Axum](https://github.com/tokio-rs/axum), featuring a complete authentication system, secure password hashing, and Docker containerization.

## 🚀 Features

- **Authentication:** Secure JWT (Access + Refresh Tokens) with rotation and blacklist revocation.
- **Security:** Argon2id password hashing, HTTP-only cookie support, and parameterized SQL queries.
- **Performance:** Asynchronous Request handling, connection pooling, and optimized Docker builds.
- **Robustness:** Structured logging (Tracing), Rate limiting, CORS, and Graceful shutdown.
- **Dockerized:** Multi-stage builds, non-root user execution, and health checks.

## 🛠️ Tech Stack

- **Language:** Rust 1.92+
- **Web Framework:** Axum 0.8
- **Database:** PostgreSQL 18.1 (Alpine)
- **ORM/Query Builder:** SQLx (Compile-time checked queries)
- **Serialization:** Serde & Serde JSON
- **Runtime:** Tokio

## 🐳 Quick Start (Docker - Recommended)

1.  **Clone the repository:**

    ```bash
    git clone https://github.com/yourusername/axum-api.git
    cd axum-api
    ```

2.  **Configure Environment:**

    ```bash
    cp .env.example .env
    # Edit .env and set a strong JWT_SECRET and POSTGRES_PASSWORD
    ```

3.  **Start Services:**

    ```bash
    docker compose up -d --build
    ```

4.  **Initialize Database:**

    ```bash
    # Run migrations inside the container
    cat migrations/*.sql | docker compose exec -T db psql -U postgres -d axum_db
    ```

5.  **Verify:**
    ```bash
    curl http://localhost:8080/
    # {"message":"Welcome to the Index API","status":"success"}
    ```

## Performance Benchmarking

To test the API performance under load, you can use **Apache Benchmark (ab)** via Docker (no installation required).

1. **Start the API:**

   ```bash
   docker compose up -d
   ```

2. **Run Benchmark:**
   (Test 500 requests with 20 concurrent users)

   ```bash
   # Linux / macOS
   docker run --rm --net=host httpd:alpine ab -n 500 -c 20 http://localhost:8080/

   # Windows (Docker Desktop)
   docker run --rm --net=host httpd:alpine ab -n 500 -c 20 http://host.docker.internal:8080/
   ```

## 💻 Local Development

1.  **Prerequisites:** Rust, built-in Postgres running locally (or via Docker).
2.  **Setup .env:** Ensure `DATABASE_URL` points to `localhost`.
3.  **Run:**

    ```bash
    # Install sqlx-cli
    cargo install sqlx-cli

    # Setup DB
    sqlx database create
    sqlx migrate run

    # Start Server
    cargo run
    ```

## 📂 Project Structure

```
src/
├── main.rs         # Application entry point & Middleware setup
├── config.rs       # Type-safe configuration from .env
├── state.rs        # Shared application state (DbPool)
├── routes/         # Route definitions (Auth, User, etc.)
├── handlers/       # Request controllers & business logic
├── models/         # Data structures & Database schemas
└── utils/          # Helpers (Hashing, JWT, Validation)
```

## 🔌 API Endpoints

| Method | Endpoint                | Description                            | Auth Required |
| :----- | :---------------------- | :------------------------------------- | :-----------: |
| GET    | `/`                     | Health Check                           |      ❌       |
| POST   | `/api/v1/auth/register` | Register new user                      |      ❌       |
| POST   | `/api/v1/auth/login`    | Login (Returns Access + Refresh Token) |      ❌       |
| POST   | `/api/v1/auth/refresh`  | Refresh Access Token                   |      ❌       |
| POST   | `/api/v1/auth/logout`   | Logout (Blacklists token)              |      ✅       |
| GET    | `/api/v1/user/profile`  | Get current user info                  |      ✅       |

## 🔒 Security Checklist for Production

Before deploying to a public server:

- [ ] Change `JWT_SECRET` to a long, random string.
- [ ] Change `POSTGRES_PASSWORD` to a strong password.
- [ ] Set `CORS_ORIGIN` to your specific frontend domain (e.g., `https://example.com`).
- [ ] Run behind a Reverse Proxy (Nginx/Traefik) with HTTPS enabled.
- [ ] Ensure database port `5432` is NOT exposed to the public internet.

## 📄 License

This project is licensed under the MIT License.
