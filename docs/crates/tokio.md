# Tokio

**Async Runtime for Rust** - Runtime สำหรับรันโค้ด Asynchronous

## 📦 ข้อมูลพื้นฐาน

| รายการ     | ข้อมูล                                                         |
| ---------- | -------------------------------------------------------------- |
| Repository | [github.com/tokio-rs/tokio](https://github.com/tokio-rs/tokio) |
| เอกสาร     | [docs.rs/tokio](https://docs.rs/tokio)                         |

## 🎯 Tokio คืออะไร?

Tokio เป็น Async Runtime ที่ช่วยให้ Rust รันโค้ด asynchronous ได้:

- **Event-driven** - ใช้ epoll/kqueue/IOCP
- **Multi-threaded** - ใช้ work-stealing scheduler
- **Performant** - รองรับล้าน connections

## 🔧 Features ที่มี

| Feature           | คำอธิบาย                              |
| ----------------- | ------------------------------------- |
| `full`            | เปิดทุก features                      |
| `rt`              | Runtime core                          |
| `rt-multi-thread` | Multi-threaded runtime                |
| `io-util`         | I/O utilities                         |
| `net`             | TCP/UDP/Unix sockets                  |
| `time`            | Timers และ delays                     |
| `sync`            | Synchronization primitives            |
| `macros`          | `#[tokio::main]` และ `#[tokio::test]` |
| `fs`              | Async file system                     |
| `process`         | Async process spawning                |
| `signal`          | Signal handling                       |

## 📝 การใช้งานพื้นฐาน

### 1. เพิ่มใน Cargo.toml

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

### 2. Basic Async Main

```rust
#[tokio::main]
async fn main() {
    println!("Hello from async!");
}
```

### 3. Spawning Tasks

```rust
use tokio::spawn;

#[tokio::main]
async fn main() {
    // Spawn background task
    let handle = spawn(async {
        // ทำงานใน background
        42
    });

    // รอผลลัพธ์
    let result = handle.await.unwrap();
    println!("Result: {}", result);
}
```

### 4. Running Tasks Concurrently

```rust
use tokio::join;

async fn task1() -> u32 { 1 }
async fn task2() -> u32 { 2 }

#[tokio::main]
async fn main() {
    // รันพร้อมกัน รอทั้งคู่เสร็จ
    let (a, b) = join!(task1(), task2());
    println!("{} + {} = {}", a, b, a + b);
}
```

### 5. Select (รอตัวแรกที่เสร็จ)

```rust
use tokio::select;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    select! {
        _ = sleep(Duration::from_secs(1)) => {
            println!("1 second passed");
        }
        _ = sleep(Duration::from_secs(2)) => {
            println!("2 seconds passed");
        }
    }
    // จะพิมพ์ "1 second passed" เพราะเสร็จก่อน
}
```

### 6. Timers และ Delays

```rust
use tokio::time::{sleep, timeout, Duration};

async fn example() {
    // หน่วงเวลา
    sleep(Duration::from_secs(1)).await;

    // Timeout
    let result = timeout(Duration::from_secs(5), long_operation()).await;
    match result {
        Ok(value) => println!("Got: {:?}", value),
        Err(_) => println!("Timed out!"),
    }
}
```

### 7. Blocking Operations (spawn_blocking)

```rust
use tokio::task::spawn_blocking;

async fn hash_password(password: String) -> String {
    // ย้ายงาน CPU-intensive ไป blocking thread pool
    spawn_blocking(move || {
        // ทำงานหนักๆ ที่จะ block thread
        expensive_hash(&password)
    }).await.unwrap()
}
```

### 8. Channels

```rust
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    // สร้าง channel
    let (tx, mut rx) = mpsc::channel(32);

    // Producer
    tokio::spawn(async move {
        tx.send("hello").await.unwrap();
        tx.send("world").await.unwrap();
    });

    // Consumer
    while let Some(msg) = rx.recv().await {
        println!("Received: {}", msg);
    }
}
```

### 9. Mutex (Async)

```rust
use tokio::sync::Mutex;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let counter = Arc::new(Mutex::new(0));

    let mut handles = vec![];
    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        handles.push(tokio::spawn(async move {
            let mut num = counter.lock().await;
            *num += 1;
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    println!("Counter: {}", *counter.lock().await);
}
```

### 10. TCP Server

```rust
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        println!("New connection from {}", addr);

        tokio::spawn(async move {
            let mut buf = [0; 1024];
            loop {
                let n = socket.read(&mut buf).await.unwrap();
                if n == 0 { return; }
                socket.write_all(&buf[0..n]).await.unwrap();
            }
        });
    }
}
```

## 📚 เมื่อไหร่ใช้อะไร

| สถานการณ์              | ใช้                  |
| ---------------------- | -------------------- |
| รัน async code         | `#[tokio::main]`     |
| รันหลาย tasks พร้อมกัน | `join!` หรือ `spawn` |
| รอตัวแรกที่เสร็จ       | `select!`            |
| งาน CPU-intensive      | `spawn_blocking`     |
| ส่งข้อมูลระหว่าง tasks | `mpsc::channel`      |
| แชร์ state             | `Arc<Mutex<T>>`      |

## 🔗 Ecosystem

- **axum** - Web framework ที่ใช้ Tokio
- **sqlx** - Async database
- **reqwest** - Async HTTP client
