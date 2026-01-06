# Axum API

![Rust](https://img.shields.io/badge/Rust-1.92-orange?logo=rust)
![Axum](https://img.shields.io/badge/Axum-0.8.8-blue)
![License](https://img.shields.io/badge/License-MIT-green)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL-18-336791?logo=postgresql&logoColor=white)

A Rust REST API built with [Axum](https://github.com/tokio-rs/axum) featuring complete authentication system.

## Features

- 🔐 **Authentication** - Register, Login, Logout
- 🎫 **JWT Tokens** - Access tokens (15 min) + Refresh tokens (7 days)
- 🔒 **Password Security** - Argon2 hashing
- 📝 **Token Blacklist** - Proper logout with token revocation
- 📊 **Structured Logging** - Tracing with configurable log levels

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [PostgreSQL](https://www.postgresql.org/download/)

## Environment Variables

Create a `.env` file:

```env
DATABASE_URL=postgres://postgres:password@localhost:5432/axum_db
HOST=0.0.0.0
PORT=8080
JWT_SECRET=your_jwt_secret_key
LOG_LEVEL=info
```

**Generate random JWT_SECRET:**

```powershell
# PowerShell (Windows)
[Convert]::ToBase64String((1..32 | ForEach-Object { Get-Random -Maximum 256 }) -as [byte[]])

# Or use OpenSSL
openssl rand -base64 32
```

## Database Setup

Run migrations in order:

```bash
psql -U postgres -d axum_db -f migrations/20260106_create_auth_users.sql
psql -U postgres -d axum_db -f migrations/20260106_create_token_blacklist.sql
psql -U postgres -d axum_db -f migrations/20260106_create_refresh_tokens.sql
```

## Development

```bash
# Run server
cargo run

# Build for release
cargo build --release
```

## API Endpoints

| Method | Endpoint           | Description                 | Auth Required |
| ------ | ------------------ | --------------------------- | ------------- |
| GET    | `/`                | Health check                | Public        |
| POST   | `/api/v1/register` | Create new user             | Public        |
| POST   | `/api/v1/login`    | Get access + refresh tokens | Public        |
| POST   | `/api/v1/refresh`  | Renew access token          | Public        |
| POST   | `/api/v1/logout`   | Revoke tokens               | 🔒 Token      |
| GET    | `/api/v1/profile`  | Protected endpoint          | 🔒 Token      |

## Usage Examples

### Register

```bash
curl -X POST http://localhost:8080/api/v1/register \
  -H "Content-Type: application/json" \
  -d '{"username":"user","password":"pass123","email":"user@example.com"}'
```

### Login

```bash
curl -X POST http://localhost:8080/api/v1/login \
  -H "Content-Type: application/json" \
  -d '{"username":"user","password":"pass123"}'
```

### Access Protected Route

```bash
curl http://localhost:8080/api/v1/profile \
  -H "Authorization: Bearer <access_token>"
```

### Refresh Token

```bash
curl -X POST http://localhost:8080/api/v1/refresh \
  -H "Content-Type: application/json" \
  -d '{"refresh_token":"<refresh_token>"}'
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
