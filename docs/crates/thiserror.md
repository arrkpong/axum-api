# Thiserror

**Error Derive Macro** - สร้าง Custom Error Types อย่างง่าย

## 📦 ข้อมูลพื้นฐาน

| รายการ     | ข้อมูล                                                               |
| ---------- | -------------------------------------------------------------------- |
| Repository | [github.com/dtolnay/thiserror](https://github.com/dtolnay/thiserror) |
| เอกสาร     | [docs.rs/thiserror](https://docs.rs/thiserror)                       |

## 🎯 Thiserror คืออะไร?

Thiserror ช่วยสร้าง Error types:

- **Derive macro** - ใช้ `#[derive(Error)]`
- **Display implementation** - สร้าง error message อัตโนมัติ
- **From implementation** - แปลง error types อัตโนมัติ

## 📝 การใช้งานพื้นฐาน

### 1. เพิ่มใน Cargo.toml

```toml
[dependencies]
thiserror = "2.0"
```

### 2. Basic Error Enum

```rust
use thiserror::Error;

#[derive(Error, Debug)]
enum AppError {
    #[error("User not found")]
    UserNotFound,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Database error")]
    DatabaseError,
}
```

### 3. Error with Context

```rust
#[derive(Error, Debug)]
enum AppError {
    #[error("User {user_id} not found")]
    UserNotFound { user_id: i32 },

    #[error("Invalid {field}: {message}")]
    ValidationError {
        field: String,
        message: String,
    },
}

// Usage
let err = AppError::UserNotFound { user_id: 123 };
println!("{}", err);  // "User 123 not found"
```

### 4. Wrapping Other Errors (From)

```rust
use std::io;
use sqlx;

#[derive(Error, Debug)]
enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found")]
    NotFound,
}

// ใช้ ? operator ได้เลย
fn read_file() -> Result<String, AppError> {
    let content = std::fs::read_to_string("file.txt")?;  // io::Error → AppError::Io
    Ok(content)
}
```

### 5. Source Error

```rust
#[derive(Error, Debug)]
enum AppError {
    #[error("Failed to connect")]
    ConnectionFailed {
        #[source]  // ใช้เป็น source() ของ error chain
        cause: io::Error,
    },
}
```

### 6. Transparent Error

```rust
#[derive(Error, Debug)]
enum AppError {
    #[error(transparent)]  // ใช้ Display และ source ของ inner error
    Other(#[from] anyhow::Error),
}
```

### 7. Multiple From Sources

```rust
#[derive(Error, Debug)]
enum DatabaseError {
    #[error("Connection failed: {0}")]
    Connection(#[from] io::Error),

    #[error("Query failed: {0}")]
    Query(#[from] sqlx::Error),

    #[error("Parse failed: {0}")]
    Parse(#[from] serde_json::Error),
}

// ทุก error type สามารถแปลงเป็น DatabaseError ได้
```

### 8. Backtrace (Rust 1.65+)

```rust
use std::backtrace::Backtrace;

#[derive(Error, Debug)]
enum AppError {
    #[error("Critical error")]
    Critical {
        #[backtrace]
        backtrace: Backtrace,
    },
}
```

### 9. Integration with Axum

```rust
use axum::{http::StatusCode, response::IntoResponse, Json};
use thiserror::Error;

#[derive(Error, Debug)]
enum ApiError {
    #[error("User not found")]
    NotFound,

    #[error("Invalid input: {0}")]
    BadRequest(String),

    #[error("Database error")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            ApiError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        (status, Json(serde_json::json!({
            "error": message
        }))).into_response()
    }
}

// ใช้ใน handler
async fn get_user(Path(id): Path<i32>) -> Result<Json<User>, ApiError> {
    let user = find_user(id).await?;  // sqlx::Error → ApiError::Database
    user.ok_or(ApiError::NotFound)
        .map(Json)
}
```

### 10. Error Hierarchy

```rust
#[derive(Error, Debug)]
enum DatabaseError {
    #[error("Connection failed")]
    Connection(#[source] io::Error),

    #[error("Query failed")]
    Query(#[source] sqlx::Error),
}

#[derive(Error, Debug)]
enum ServiceError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("Validation error: {0}")]
    Validation(String),
}

#[derive(Error, Debug)]
enum ApiError {
    #[error("{0}")]
    Service(#[from] ServiceError),

    #[error("Unauthorized")]
    Unauthorized,
}
```

## 📚 Attributes

| Attribute               | หน้าที่                    |
| ----------------------- | -------------------------- |
| `#[error("...")]`       | กำหนด Display message      |
| `#[from]`               | Implement From trait       |
| `#[source]`             | ใช้เป็น error source       |
| `#[backtrace]`          | Capture backtrace          |
| `#[error(transparent)]` | Forward Display และ source |

## 📝 Error Message Formatting

```rust
#[derive(Error, Debug)]
enum MyError {
    // Static message
    #[error("Something went wrong")]
    Simple,

    // With positional args
    #[error("Error code: {0}")]
    WithCode(i32),

    // With named fields
    #[error("User {name} (id={id}) not found")]
    UserNotFound { id: i32, name: String },

    // Using Display of inner
    #[error("IO error: {0}")]
    Io(io::Error),

    // Using Debug of inner
    #[error("Parse error: {0:?}")]
    Parse(serde_json::Error),
}
```

## 🔗 Ecosystem

- **anyhow** - Dynamic error handling (ใช้คู่กัน)
- **axum** - Web framework error handling
- **sqlx** - Database errors
