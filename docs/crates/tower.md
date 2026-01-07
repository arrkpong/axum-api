# Tower

**Middleware Framework** - เฟรมเวิร์คสำหรับสร้าง Middleware

## 📦 ข้อมูลพื้นฐาน

| รายการ     | ข้อมูล                                                         |
| ---------- | -------------------------------------------------------------- |
| Repository | [github.com/tower-rs/tower](https://github.com/tower-rs/tower) |
| เอกสาร     | [docs.rs/tower](https://docs.rs/tower)                         |

## 🎯 Tower คืออะไร?

Tower เป็น Middleware Framework ที่ช่วยจัดการ:

- **Service abstraction** - นามธรรมสำหรับ request/response
- **Composable middleware** - ประกอบ middleware เป็นชั้นๆ
- **Reusable components** - ใช้ซ้ำได้กับหลาย frameworks

## 🔧 Features ที่มี

| Feature       | คำอธิบาย                            |
| ------------- | ----------------------------------- |
| `full`        | เปิดทุก features                    |
| `util`        | ServiceBuilder และ utilities        |
| `limit`       | Rate limiting และ concurrency limit |
| `timeout`     | Request timeout                     |
| `buffer`      | Request buffering                   |
| `retry`       | Automatic retry                     |
| `load`        | Load balancing                      |
| `discover`    | Service discovery                   |
| `hedge`       | Hedged requests                     |
| `filter`      | Request filtering                   |
| `spawn-ready` | Spawn service readiness checks      |

## 📝 การใช้งานพื้นฐาน

### 1. เพิ่มใน Cargo.toml

```toml
[dependencies]
tower = { version = "0.5", features = ["full"] }
```

### 2. Service Trait

```rust
use tower::Service;
use std::task::{Context, Poll};

// Service trait คือหัวใจของ Tower
pub trait Service<Request> {
    type Response;
    type Error;
    type Future: Future<Output = Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;
    fn call(&mut self, req: Request) -> Self::Future;
}
```

### 3. ServiceBuilder

```rust
use tower::ServiceBuilder;
use tower::limit::RateLimitLayer;
use tower::timeout::TimeoutLayer;
use std::time::Duration;

let service = ServiceBuilder::new()
    .layer(TimeoutLayer::new(Duration::from_secs(30)))
    .layer(RateLimitLayer::new(100, Duration::from_secs(1)))
    .service(my_service);
```

### 4. RateLimitLayer

```rust
use tower::limit::RateLimitLayer;
use std::time::Duration;

// อนุญาต 100 requests ต่อ 1 วินาที
let layer = RateLimitLayer::new(100, Duration::from_secs(1));
```

### 5. ConcurrencyLimitLayer

```rust
use tower::limit::ConcurrencyLimitLayer;

// อนุญาต max 10 concurrent requests
let layer = ConcurrencyLimitLayer::new(10);
```

### 6. TimeoutLayer

```rust
use tower::timeout::TimeoutLayer;
use std::time::Duration;

// ตัด request ที่นานเกิน 30 วินาที
let layer = TimeoutLayer::new(Duration::from_secs(30));
```

### 7. BufferLayer

```rust
use tower::buffer::BufferLayer;

// สร้าง buffer ขนาด 1024 requests
let layer = BufferLayer::new(1024);
```

### 8. RetryLayer

```rust
use tower::retry::{RetryLayer, Policy};

// กำหนด retry policy
struct MyPolicy;

impl<E> Policy<Request, Response, E> for MyPolicy {
    type Future = future::Ready<Self>;

    fn retry(&self, _: &Request, result: Result<&Response, &E>) -> Option<Self::Future> {
        match result {
            Ok(_) => None,  // สำเร็จ ไม่ต้อง retry
            Err(_) => Some(future::ready(MyPolicy)),  // retry
        }
    }

    fn clone_request(&self, req: &Request) -> Option<Request> {
        Some(req.clone())
    }
}

let layer = RetryLayer::new(MyPolicy);
```

### 9. Layer Ordering (สำคัญมาก!)

```rust
// Layers ทำงานแบบ wrap จากล่างขึ้นบน
ServiceBuilder::new()
    .layer(OuterLayer)    // ทำงานก่อนสุดขาเข้า, หลังสุดขาออก
    .layer(MiddleLayer)
    .layer(InnerLayer)    // ทำงานหลังสุดขาเข้า, ก่อนสุดขาออก
    .service(inner_service)
```

**Request Flow:**

```
Request → Outer → Middle → Inner → Service
                                      ↓
Response ← Outer ← Middle ← Inner ← Result
```

### 10. Custom Layer

```rust
use tower::{Layer, Service};

struct LoggingLayer;

impl<S> Layer<S> for LoggingLayer {
    type Service = LoggingService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LoggingService { inner }
    }
}

struct LoggingService<S> {
    inner: S,
}

impl<S, Request> Service<Request> for LoggingService<S>
where
    S: Service<Request>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        println!("Request received");
        self.inner.call(req)
    }
}
```

## 📚 Middleware ที่มี

| Layer                   | หน้าที่                   |
| ----------------------- | ------------------------- |
| `RateLimitLayer`        | จำกัด requests/เวลา       |
| `ConcurrencyLimitLayer` | จำกัด concurrent requests |
| `TimeoutLayer`          | ตัด request ที่นานเกิน    |
| `BufferLayer`           | Queue requests            |
| `RetryLayer`            | Retry เมื่อ error         |
| `LoadShedLayer`         | ปฏิเสธเมื่อ overloaded    |

## 🔗 Ecosystem

- **tower-http** - HTTP-specific middleware
- **axum** - Web framework ที่ใช้ Tower
- **tonic** - gRPC framework ที่ใช้ Tower
