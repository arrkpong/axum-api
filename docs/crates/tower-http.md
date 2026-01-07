# Tower HTTP

**HTTP Middleware for Tower** - HTTP Middleware สำหรับ Tower

## 📦 ข้อมูลพื้นฐาน

| รายการ     | ข้อมูล                                                                   |
| ---------- | ------------------------------------------------------------------------ |
| Repository | [github.com/tower-rs/tower-http](https://github.com/tower-rs/tower-http) |
| เอกสาร     | [docs.rs/tower-http](https://docs.rs/tower-http)                         |

## 🎯 Tower HTTP คืออะไร?

Tower HTTP เป็น Middleware ที่ออกแบบมาเฉพาะสำหรับ HTTP:

- **CORS** - Cross-Origin Resource Sharing
- **Compression** - Response compression
- **Tracing** - Request/Response logging
- **Headers** - Header manipulation

## 🔧 Features ที่มี

| Feature              | คำอธิบาย                                     |
| -------------------- | -------------------------------------------- |
| `full`               | เปิดทุก features                             |
| `cors`               | CORS support                                 |
| `trace`              | Request tracing                              |
| `timeout`            | Request timeout                              |
| `compression-full`   | ทุก compression: Gzip, Brotli, Deflate, Zstd |
| `compression-gzip`   | Gzip only                                    |
| `compression-br`     | Brotli only                                  |
| `decompression-full` | Request decompression                        |
| `sensitive-headers`  | Hide sensitive headers in logs               |
| `request-id`         | Generate request IDs                         |
| `limit`              | Request body limit                           |
| `fs`                 | Static file serving                          |
| `catch-panic`        | Catch panics                                 |

## 📝 การใช้งานพื้นฐาน

### 1. เพิ่มใน Cargo.toml

```toml
[dependencies]
tower-http = { version = "0.6", features = ["full"] }
```

### 2. TraceLayer (Logging)

```rust
use tower_http::trace::TraceLayer;

let app = Router::new()
    .route("/", get(handler))
    .layer(TraceLayer::new_for_http());
```

**ผลลัพธ์ใน Log:**

```
INFO  request{method=GET uri=/api/users} started processing
INFO  request{method=GET uri=/api/users} response_code=200 latency=5ms
```

**Custom Tracing:**

```rust
use tower_http::trace::{TraceLayer, DefaultOnRequest, DefaultOnResponse};
use tracing::Level;

let layer = TraceLayer::new_for_http()
    .on_request(DefaultOnRequest::new().level(Level::INFO))
    .on_response(DefaultOnResponse::new().level(Level::INFO).latency_unit(LatencyUnit::Millis));
```

### 3. CorsLayer

```rust
use tower_http::cors::CorsLayer;

// อนุญาตทุก origin (development only!)
let cors = CorsLayer::permissive();

// กำหนด origin เฉพาะ (production)
use tower_http::cors::{Any, CorsLayer};
use http::{Method, HeaderValue};

let cors = CorsLayer::new()
    .allow_origin("https://example.com".parse::<HeaderValue>().unwrap())
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers(Any);
```

### 4. CompressionLayer

```rust
use tower_http::compression::CompressionLayer;

// เปิด compression อัตโนมัติ (Gzip, Brotli, Deflate)
let app = Router::new()
    .route("/", get(handler))
    .layer(CompressionLayer::new());
```

**กำหนด compression เฉพาะ:**

```rust
use tower_http::compression::predicate::{NotForContentType, SizeAbove};

let layer = CompressionLayer::new()
    .compress_when(SizeAbove::new(1024));  // compress เมื่อ > 1KB
```

### 5. DecompressionLayer

```rust
use tower_http::decompression::RequestDecompressionLayer;

// แตก compressed request body อัตโนมัติ
let app = Router::new()
    .layer(RequestDecompressionLayer::new());
```

### 6. TimeoutLayer

```rust
use tower_http::timeout::TimeoutLayer;
use std::time::Duration;

let layer = TimeoutLayer::new(Duration::from_secs(30));
```

### 7. RequestIdLayer

```rust
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};

let app = Router::new()
    .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
    .layer(PropagateRequestIdLayer::x_request_id());
```

### 8. SensitiveHeadersLayer

```rust
use tower_http::sensitive_headers::SetSensitiveHeadersLayer;
use http::header::{AUTHORIZATION, COOKIE};

// ซ่อน headers เหล่านี้ใน logs
let layer = SetSensitiveHeadersLayer::new([AUTHORIZATION, COOKIE]);
```

### 9. LimitRequestBodyLayer

```rust
use tower_http::limit::RequestBodyLimitLayer;

// จำกัด request body ไม่เกิน 10MB
let layer = RequestBodyLimitLayer::new(10 * 1024 * 1024);
```

### 10. CatchPanicLayer

```rust
use tower_http::catch_panic::CatchPanicLayer;

// จับ panic แล้วคืน 500 แทนที่จะ crash
let app = Router::new()
    .layer(CatchPanicLayer::new());
```

### 11. ServeDir (Static Files)

```rust
use tower_http::services::ServeDir;

let app = Router::new()
    .nest_service("/static", ServeDir::new("public"));
```

### 12. SetResponseHeaderLayer

```rust
use tower_http::set_header::SetResponseHeaderLayer;
use http::{header, HeaderValue};

let layer = SetResponseHeaderLayer::if_not_present(
    header::X_CONTENT_TYPE_OPTIONS,
    HeaderValue::from_static("nosniff"),
);
```

## 📚 Middleware ที่มี (สรุป)

| Layer                   | หน้าที่                |
| ----------------------- | ---------------------- |
| `TraceLayer`            | Log requests/responses |
| `CorsLayer`             | CORS handling          |
| `CompressionLayer`      | Compress responses     |
| `DecompressionLayer`    | Decompress requests    |
| `TimeoutLayer`          | Request timeout        |
| `RequestIdLayer`        | Generate request IDs   |
| `SensitiveHeadersLayer` | Hide sensitive headers |
| `LimitRequestBodyLayer` | Limit request size     |
| `CatchPanicLayer`       | Catch panics           |
| `ServeDir`              | Serve static files     |
| `SetHeaderLayer`        | Add/modify headers     |

## 🔗 Ecosystem

- **tower** - Core middleware framework
- **axum** - Web framework
- **tracing** - Logging system
