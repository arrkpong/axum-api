# Tracing

**Structured Logging** - ระบบ Logging แบบ Structured

## 📦 ข้อมูลพื้นฐาน

| รายการ     | ข้อมูล                                                             |
| ---------- | ------------------------------------------------------------------ |
| Repository | [github.com/tokio-rs/tracing](https://github.com/tokio-rs/tracing) |
| เอกสาร     | [docs.rs/tracing](https://docs.rs/tracing)                         |

## 🎯 Tracing คืออะไร?

Tracing เป็น Logging framework ที่:

- **Structured** - Log มี key-value pairs
- **Leveled** - แบ่งระดับ severity
- **Span-based** - ติดตาม request lifecycle
- **Async-aware** - ทำงานถูกต้องใน async code

## 🔧 Components

| Crate                   | หน้าที่                    |
| ----------------------- | -------------------------- |
| `tracing`               | Core macros และ types      |
| `tracing-subscriber`    | Subscriber implementations |
| `tracing-appender`      | File appender              |
| `tracing-opentelemetry` | OpenTelemetry integration  |

## 📝 การใช้งานพื้นฐาน

### 1. เพิ่มใน Cargo.toml

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

### 2. Setup Subscriber

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn init_logging() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

fn main() {
    init_logging();
    // ...
}
```

### 3. Log Levels

```rust
use tracing::{error, warn, info, debug, trace};

error!("Critical error occurred");   // สำคัญที่สุด
warn!("Something suspicious");
info!("Normal information");
debug!("Debug information");
trace!("Detailed trace");            // ละเอียดที่สุด
```

### 4. Structured Logging

```rust
use tracing::info;

let user_id = 123;
let action = "login";

// Field syntax
info!(user_id, action, "User performed action");
// Output: INFO User performed action user_id=123 action="login"

// Display format
info!(%user_id, %action, "User {} performed {}", user_id, action);

// Debug format
info!(?some_struct, "Complex data");
```

### 5. Spans (ติดตาม Lifecycle)

```rust
use tracing::{info_span, Instrument};

async fn handle_request(request_id: &str) {
    let span = info_span!("request", %request_id);

    async {
        info!("Processing started");
        // ... do work
        info!("Processing completed");
    }
    .instrument(span)
    .await;
}
// Output:
// INFO request{request_id="abc123"} Processing started
// INFO request{request_id="abc123"} Processing completed
```

### 6. #[instrument] Attribute

```rust
use tracing::instrument;

#[instrument(skip(password))]  // ไม่ log password
async fn login(username: &str, password: &str) -> Result<User, Error> {
    info!("Attempting login");
    // ...
}
// Output: INFO login{username="john"} Attempting login
```

### 7. Span Fields

```rust
use tracing::{info_span, Span};

fn process() {
    let span = info_span!("process", items = tracing::field::Empty);
    let _enter = span.enter();

    // เพิ่ม field ทีหลัง
    Span::current().record("items", 42);

    info!("Processing");
}
```

### 8. Error Logging

```rust
use tracing::error;

fn handle_error(err: &dyn std::error::Error) {
    error!(error = %err, "Operation failed");

    // หรือใช้ source chain
    error!(error = ?err, "Operation failed with debug");
}
```

### 9. Custom Subscriber

```rust
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn init_logging() {
    let fmt_layer = fmt::layer()
        .with_target(true)           // แสดง module path
        .with_thread_ids(true)       // แสดง thread ID
        .with_file(true)             // แสดง file name
        .with_line_number(true)      // แสดง line number
        .compact();                  // output แบบกระชับ

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt_layer)
        .init();
}
```

### 10. JSON Output

```rust
use tracing_subscriber::fmt::format::json;

let fmt_layer = fmt::layer()
    .json();  // Output เป็น JSON

// Output: {"timestamp":"...","level":"INFO","message":"Hello","user_id":123}
```

### 11. File Logging

```rust
use tracing_appender::rolling::{RollingFileAppender, Rotation};

let file_appender = RollingFileAppender::new(
    Rotation::DAILY,
    "logs",
    "app.log",
);

let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

tracing_subscriber::fmt()
    .with_writer(non_blocking)
    .init();
```

### 12. Multiple Outputs

```rust
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

// Log ไป terminal และ file พร้อมกัน
let file_appender = tracing_appender::rolling::daily("logs", "app.log");
let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);

tracing_subscriber::registry()
    .with(fmt::layer().with_writer(std::io::stdout))
    .with(fmt::layer().with_writer(file_writer).json())
    .init();
```

## 📚 Format Specifiers

| Specifier | หน้าที่             |
| --------- | ------------------- |
| `%var`    | ใช้ `Display` trait |
| `?var`    | ใช้ `Debug` trait   |
| `var`     | ใช้ค่าตรงๆ          |

## 🔧 Environment Variables

```bash
# กำหนด log level
RUST_LOG=info
RUST_LOG=debug
RUST_LOG=myapp=debug,other=warn

# กำหนดเฉพาะ module
RUST_LOG=myapp::handlers=trace,sqlx=warn
```

## 🔗 Ecosystem

- **tower-http** - TraceLayer
- **axum** - Request tracing
- **sqlx** - Query logging
- **tracing-opentelemetry** - Distributed tracing
