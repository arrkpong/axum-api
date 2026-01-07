# SQLx

**Async SQL Toolkit** - เครื่องมือเชื่อมต่อ Database แบบ Async

## 📦 ข้อมูลพื้นฐาน

| รายการ     | ข้อมูล                                                             |
| ---------- | ------------------------------------------------------------------ |
| Repository | [github.com/launchbadge/sqlx](https://github.com/launchbadge/sqlx) |
| เอกสาร     | [docs.rs/sqlx](https://docs.rs/sqlx)                               |

## 🎯 SQLx คืออะไร?

SQLx เป็น Async SQL toolkit ที่:

- **Compile-time checked** - ตรวจสอบ SQL ตอน compile
- **Async-native** - รองรับ async/await
- **Pure Rust** - ไม่ต้องติดตั้ง C libraries
- **Multiple databases** - PostgreSQL, MySQL, SQLite

## 🔧 Features ที่มี

| Feature                | คำอธิบาย                        |
| ---------------------- | ------------------------------- |
| `runtime-tokio`        | ใช้ Tokio runtime               |
| `runtime-async-std`    | ใช้ async-std runtime           |
| `runtime-tokio-rustls` | Tokio + RustTLS                 |
| `postgres`             | PostgreSQL support              |
| `mysql`                | MySQL support                   |
| `sqlite`               | SQLite support                  |
| `macros`               | `query!` และ `query_as!` macros |
| `uuid`                 | UUID type support               |
| `chrono`               | DateTime type support           |
| `json`                 | JSON type support               |
| `migrate`              | Database migrations             |

## 📝 การใช้งานพื้นฐาน

### 1. เพิ่มใน Cargo.toml

```toml
[dependencies]
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "macros"] }
```

### 2. สร้าง Connection Pool

```rust
use sqlx::postgres::PgPoolOptions;

let pool = PgPoolOptions::new()
    .max_connections(5)
    .connect("postgres://user:pass@localhost/dbname")
    .await?;
```

### 3. Query with query! macro

```rust
// Compile-time checked query
let users = sqlx::query!(
    "SELECT id, name, email FROM users WHERE active = $1",
    true
)
.fetch_all(&pool)
.await?;

for user in users {
    println!("{}: {}", user.id, user.name);
}
```

### 4. Query with query_as! macro

```rust
struct User {
    id: i32,
    name: String,
    email: Option<String>,
}

let user = sqlx::query_as!(
    User,
    "SELECT id, name, email FROM users WHERE id = $1",
    user_id
)
.fetch_optional(&pool)
.await?;
```

### 5. Fetch Methods

```rust
// คืน 1 row (error ถ้าไม่มี)
let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
    .fetch_one(&pool)
    .await?;

// คืน Option<T>
let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
    .fetch_optional(&pool)
    .await?;

// คืน Vec<T>
let users = sqlx::query_as!(User, "SELECT * FROM users")
    .fetch_all(&pool)
    .await?;

// Stream
let mut rows = sqlx::query_as!(User, "SELECT * FROM users")
    .fetch(&pool);
while let Some(user) = rows.try_next().await? {
    println!("{}", user.name);
}
```

### 6. Execute (INSERT/UPDATE/DELETE)

```rust
// Insert
let result = sqlx::query!(
    "INSERT INTO users (name, email) VALUES ($1, $2)",
    "สมชาย",
    "somchai@example.com"
)
.execute(&pool)
.await?;

println!("Rows affected: {}", result.rows_affected());

// Insert with RETURNING
let user = sqlx::query_as!(
    User,
    "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING *",
    "สมชาย",
    "somchai@example.com"
)
.fetch_one(&pool)
.await?;
```

### 7. Transactions

```rust
let mut tx = pool.begin().await?;

sqlx::query!("UPDATE accounts SET balance = balance - $1 WHERE id = $2", 100, from_id)
    .execute(&mut *tx)
    .await?;

sqlx::query!("UPDATE accounts SET balance = balance + $1 WHERE id = $2", 100, to_id)
    .execute(&mut *tx)
    .await?;

tx.commit().await?;
// หรือ tx.rollback().await? เพื่อยกเลิก
```

### 8. Dynamic Queries (QueryBuilder)

```rust
use sqlx::QueryBuilder;

let mut builder = QueryBuilder::new("SELECT * FROM users WHERE 1=1");

if let Some(name) = name_filter {
    builder.push(" AND name = ").push_bind(name);
}

if let Some(active) = active_filter {
    builder.push(" AND active = ").push_bind(active);
}

let users = builder.build_query_as::<User>()
    .fetch_all(&pool)
    .await?;
```

### 9. Raw Query (ไม่ใช้ macro)

```rust
use sqlx::{query, query_as, Row};

// ใช้ Row trait
let row = query("SELECT id, name FROM users WHERE id = $1")
    .bind(user_id)
    .fetch_one(&pool)
    .await?;

let id: i32 = row.get("id");
let name: String = row.get("name");

// ใช้กับ FromRow
#[derive(sqlx::FromRow)]
struct User {
    id: i32,
    name: String,
}

let user: User = query_as("SELECT * FROM users WHERE id = $1")
    .bind(user_id)
    .fetch_one(&pool)
    .await?;
```

### 10. Migrations

```bash
# ติดตั้ง CLI
cargo install sqlx-cli

# สร้าง migration
sqlx migrate add create_users_table

# รัน migrations
sqlx migrate run

# Revert
sqlx migrate revert
```

**Migration file example:**

```sql
-- migrations/20240101_create_users_table.sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

**รันใน code:**

```rust
sqlx::migrate!("./migrations")
    .run(&pool)
    .await?;
```

## 📚 Type Mappings (PostgreSQL)

| PostgreSQL        | Rust                    |
| ----------------- | ----------------------- |
| `INTEGER`         | `i32`                   |
| `BIGINT`          | `i64`                   |
| `TEXT`, `VARCHAR` | `String`                |
| `BOOLEAN`         | `bool`                  |
| `TIMESTAMPTZ`     | `chrono::DateTime<Utc>` |
| `UUID`            | `uuid::Uuid`            |
| `JSONB`           | `serde_json::Value`     |

## 🛡️ SQL Injection Protection

```rust
// ✅ Safe - parameterized
sqlx::query!("SELECT * FROM users WHERE id = $1", user_id)

// ❌ Dangerous - string concatenation
let query = format!("SELECT * FROM users WHERE id = {}", user_id);
```

## 🔗 Ecosystem

- **tokio** - Async runtime
- **chrono** - DateTime
- **uuid** - UUID
- **serde_json** - JSON columns
