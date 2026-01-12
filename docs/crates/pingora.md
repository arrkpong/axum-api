# Pingora

**High Performance Proxy** - ไลบรารีสำหรับสร้าง Proxy, Load Balancer, และ Gateway ประสิทธิภาพสูง

## 📦 ข้อมูลพื้นฐาน

| รายการ     | ข้อมูล                                                                 |
| ---------- | ---------------------------------------------------------------------- |
| Repository | [github.com/cloudflare/pingora](https://github.com/cloudflare/pingora) |
| เอกสาร     | [docs.rs/pingora](https://docs.rs/pingora)                             |

## 🎯 Pingora คืออะไร?

Pingora คือ Rust Framework ที่ **Cloudflare** สร้างขึ้นเพื่อใช้แทน Nginx ในระบบภายในของเขา โดยเน้นที่:

- **Safety** - ปลอดภัยจาก Memory Safety bug (Buffer overflow ฯลฯ)
- **Performance** - ประสิทธิภาพสูงมาก โดยเฉพาะเมื่อรับโหลดหนักๆ (High Load)
- **Programmable** - เขียน Logic การ Routing/Filtering ด้วยภาษา Rust ได้ละเอียด (ไม่ได้แก้แค่ Config file)

## 🔧 Features ที่ใช้

ในโปรเจคนี้เราใช้ Module: `pingora-proxy` และ `pingora-load-balancing`

| Feature  | คำอธิบาย                                        |
| -------- | ----------------------------------------------- |
| `lb`     | Load Balancer (Round-Robin, Consistent Hashing) |
| `proxy`  | HTTP/TCP Proxy Logic                            |
| `server` | ตัว Web Server process                          |

## 📝 การใช้งานในโปรเจคนี้ (Reverse Proxy)

เราสร้างโปรเจคย่อยชื่อ `pingora_proxy` เพื่อทำหน้าที่เป็น **Entry Point** (Port 80) รับ Traffic จากโลกภายนอก แล้วส่งต่อให้ **Axum API** (Port 8080)

### 1. โครงสร้าง Dependencies (Cargo.toml)

**สำคัญ:** ต้องใช้ Version `0.6+` เพื่อรองรับ Build Toolchain สมัยใหม่

```toml
[dependencies]
pingora = { version = "0.6.0", features = ["lb"] }
async-trait = "0.1"
tokio = { version = "1.0", features = ["full"] }
```

### 2. Logic การทำงาน (src/main.rs)

1.  **Server Initialization:** สร้าง Pingora Server
2.  **Upstream Selection:** กำหนดเป้าหมาย (Axum API) ผ่าน Docker DNS (`api:8080`)
3.  **Listening:** เปิด Port 80 รอรับ Request

```rust
// ตัวอย่าง Logic การส่งต่อ Request
async fn upstream_peer(&self, _session: &mut Session, _ctx: &mut Self::CTX) -> Result<Box<HttpPeer>> {
    // เลือก API Server ปลายทาง
    let upstream = self.lb.select(b"", 256).unwrap();

    // สร้าง Connection ไปยัง Web Container (ไม่ใช้ SSL เพราะคุยกันในวง Docker Network)
    let peer = Box::new(HttpPeer::new(upstream, false, "api".to_string()));
    Ok(peer)
}
```

## 🏗️ การ Build & Deploy

เนื่องจาก Pingora มี C Dependencies (BoringSSL, CMake, Clang) ที่ซับซ้อน เราจึงต้องใช้ **Docker Multi-stage Build**

### Dockerfile Highlights

```dockerfile
# 1. Builder Stage (Rust Full Image)
FROM rust:1.92.0-bookworm AS builder
# ลง Tools จำเป็น: cmake, clang, perl, golang, nasm
RUN apt-get install -y cmake clang libssl-dev protobuf-compiler pkg-config build-essential perl nasm golang-go

# 2. Runtime Stage (Ubuntu Slim/LTS)
FROM ubuntu:22.04
# ก๊อปปี้ Binary ที่ Compile เสร็จแล้วมาวาง
COPY --from=builder /usr/src/app/target/release/pingora_proxy /usr/local/bin/pingora_proxy
Entrypoint ["pingora_proxy"]
```

## ⚠️ ข้อควรระวัง

1.  **Windows Incompatibility:** Pingora รันบน Windows โดยตรงไม่ได้ (ต้องใช้ Linux/Docker เท่านั้น) เพราะใช้ `epoll`
2.  **Compilation Time:** ใช้เวลา Compile นาน (2-5 นาที) ในครั้งแรก เพราะต้อง Build BoringSSL
3.  **Optimization:** แนะนำให้เปิด `[profile.release]` ใน `Cargo.toml` เพื่อลดขนาด Image

    ```toml
    [profile.release]
    strip = true
    lto = true
    opt-level = 3
    codegen-units = 1
    ```
