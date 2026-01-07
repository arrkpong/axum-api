# Axum

**Web Framework for Rust** - เฟรมเวิร์คสำหรับสร้าง Web API

## 📦 ข้อมูลพื้นฐาน

| รายการ     | ข้อมูล                                                       |
| ---------- | ------------------------------------------------------------ |
| Repository | [github.com/tokio-rs/axum](https://github.com/tokio-rs/axum) |
| เอกสาร     | [docs.rs/axum](https://docs.rs/axum)                         |

## 🎯 Axum คืออะไร?

Axum เป็น Web Framework ที่สร้างโดยทีม Tokio ออกแบบมาเพื่อ:

- **Ergonomic** - ใช้งานง่าย เขียนโค้ดสั้น
- **Modular** - ใช้ Tower ecosystem สำหรับ middleware
- **Type-safe** - ตรวจสอบ type ตอน compile
- **Async-first** - รองรับ async/await เต็มรูปแบบ

## 🔧 Features ที่มี

| Feature        | คำอธิบาย                                                   |
| -------------- | ---------------------------------------------------------- |
| `macros`       | เปิดใช้ `#[debug_handler]` สำหรับ error messages ที่ชัดเจน |
| `ws`           | WebSocket support                                          |
| `multipart`    | รองรับ multipart form data (file upload)                   |
| `matched-path` | เข้าถึง matched route path                                 |
| `original-uri` | เข้าถึง original URI ก่อน routing                          |
| `query`        | Query string parsing                                       |
| `form`         | Form data parsing                                          |
| `json`         | JSON body parsing (เปิดโดย default)                        |

## 📝 การใช้งานพื้นฐาน

### 1. เพิ่มใน Cargo.toml

```toml
[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
```

### 2. สร้าง Hello World

```rust
use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### 3. Routing

```rust
use axum::{Router, routing::{get, post, put, delete}};

let app = Router::new()
    .route("/", get(index))
    .route("/users", get(list_users).post(create_user))
    .route("/users/:id", get(get_user).put(update_user).delete(delete_user));
```

### 4. Path Parameters

```rust
use axum::extract::Path;

// GET /users/123
async fn get_user(Path(id): Path<u64>) -> String {
    format!("User ID: {}", id)
}

// GET /users/123/posts/456
async fn get_post(Path((user_id, post_id)): Path<(u64, u64)>) -> String {
    format!("User {} Post {}", user_id, post_id)
}
```

### 5. Query Parameters

```rust
use axum::extract::Query;
use serde::Deserialize;

#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    limit: Option<u32>,
}

// GET /users?page=1&limit=10
async fn list_users(Query(params): Query<Pagination>) -> String {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(10);
    format!("Page {} with {} items", page, limit)
}
```

### 6. JSON Request/Response

```rust
use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct CreateUser {
    username: String,
    email: String,
}

#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}

async fn create_user(Json(payload): Json<CreateUser>) -> Json<User> {
    let user = User {
        id: 1,
        username: payload.username,
    };
    Json(user)
}
```

### 7. State (แชร์ข้อมูลระหว่าง Handlers)

```rust
use axum::extract::State;
use std::sync::Arc;

struct AppState {
    db_pool: PgPool,
    config: Config,
}

let state = Arc::new(AppState { ... });

let app = Router::new()
    .route("/users", get(list_users))
    .with_state(state);

async fn list_users(State(state): State<Arc<AppState>>) -> ... {
    // ใช้ state.db_pool ได้
}
```

### 8. Custom Status Code

```rust
use axum::http::StatusCode;

async fn create_user(...) -> (StatusCode, Json<User>) {
    (StatusCode::CREATED, Json(user))
}

async fn not_found() -> StatusCode {
    StatusCode::NOT_FOUND
}
```

### 9. Error Handling

```rust
use axum::{http::StatusCode, response::IntoResponse};

enum AppError {
    NotFound,
    InternalError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found").into_response(),
            AppError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Error").into_response(),
        }
    }
}

async fn get_user(Path(id): Path<u64>) -> Result<Json<User>, AppError> {
    let user = find_user(id).ok_or(AppError::NotFound)?;
    Ok(Json(user))
}
```

### 10. Middleware (Layer)

```rust
use tower_http::trace::TraceLayer;
use tower_http::cors::CorsLayer;

let app = Router::new()
    .route("/", get(index))
    .layer(TraceLayer::new_for_http())
    .layer(CorsLayer::permissive());
```

## 📚 Extractors ที่มี

| Extractor      | หน้าที่                |
| -------------- | ---------------------- |
| `Path<T>`      | ดึงค่าจาก URL path     |
| `Query<T>`     | ดึงค่าจาก query string |
| `Json<T>`      | Parse JSON body        |
| `Form<T>`      | Parse form data        |
| `State<T>`     | เข้าถึง shared state   |
| `Headers`      | เข้าถึง HTTP headers   |
| `Extension<T>` | เข้าถึง extension data |
| `Request`      | เข้าถึง raw request    |

## 🔗 Ecosystem

- **tower** - Middleware framework
- **tower-http** - HTTP middleware (CORS, Compression, Tracing)
- **axum-extra** - เพิ่มเติม extractors และ utilities
