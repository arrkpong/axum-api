# Crate Documentation Index

เอกสารอธิบายการใช้งาน Rust Crates พื้นฐาน (ภาษาไทย)

## 📚 รายการเอกสาร

### Web Framework

| Crate     | หน้าที่       | เอกสาร                 |
| --------- | ------------- | ---------------------- |
| **axum**  | Web Framework | [axum.md](./axum.md)   |
| **tokio** | Async Runtime | [tokio.md](./tokio.md) |

### Middleware

| Crate          | หน้าที่              | เอกสาร                           |
| -------------- | -------------------- | -------------------------------- |
| **tower**      | Middleware Framework | [tower.md](./tower.md)           |
| **tower-http** | HTTP Middleware      | [tower-http.md](./tower-http.md) |

### Database

| Crate    | หน้าที่           | เอกสาร               |
| -------- | ----------------- | -------------------- |
| **sqlx** | Async SQL Toolkit | [sqlx.md](./sqlx.md) |

### Serialization

| Crate     | หน้าที่                 | เอกสาร                 |
| --------- | ----------------------- | ---------------------- |
| **serde** | Serialization Framework | [serde.md](./serde.md) |

### Security

| Crate            | หน้าที่            | เอกสาร                                                   |
| ---------------- | ------------------ | -------------------------------------------------------- |
| **argon2**       | Password Hashing   | [security.md](./security.md#argon2)                      |
| **jsonwebtoken** | JWT Authentication | [security.md](./security.md#json-web-token-jsonwebtoken) |

### Utilities

| Crate       | หน้าที่               | เอกสาร                     |
| ----------- | --------------------- | -------------------------- |
| **chrono**  | Date & Time           | [chrono.md](./chrono.md)   |
| **uuid**    | Unique IDs            | [uuid.md](./uuid.md)       |
| **dotenvy** | Environment Variables | [dotenvy.md](./dotenvy.md) |

### Validation & Error Handling

| Crate         | หน้าที่            | เอกสาร                         |
| ------------- | ------------------ | ------------------------------ |
| **validator** | Input Validation   | [validator.md](./validator.md) |
| **thiserror** | Custom Error Types | [thiserror.md](./thiserror.md) |

### Documentation

| Crate      | หน้าที่         | เอกสาร                   |
| ---------- | --------------- | ------------------------ |
| **utoipa** | OpenAPI/Swagger | [utoipa.md](./utoipa.md) |

### Logging

| Crate       | หน้าที่            | เอกสาร                     |
| ----------- | ------------------ | -------------------------- |
| **tracing** | Structured Logging | [tracing.md](./tracing.md) |

---

## 🔗 Dependency Graph

```
axum
├── tokio (runtime)
├── tower (middleware)
│   └── tower-http
├── serde (json)
├── sqlx (database)
│   ├── chrono
│   └── uuid
└── tracing (logging)

Security
├── argon2 (password hashing)
└── jsonwebtoken (authentication)

Tools
├── dotenvy (env loader)
└── utoipa (api docs)
```

---

## 📖 วิธีใช้เอกสารนี้

1. **เริ่มต้น** → อ่าน axum.md และ tokio.md
2. **Database** → อ่าน sqlx.md
3. **Security** → อ่าน security.md
4. **Advanced** → อ่าน tower.md และ tower-http.md

---

## 🔗 External Resources

| Resource      | Link                                                                                     |
| ------------- | ---------------------------------------------------------------------------------------- |
| Rust Book     | [doc.rust-lang.org/book](https://doc.rust-lang.org/book/)                                |
| Crates.io     | [crates.io](https://crates.io/)                                                          |
| Docs.rs       | [docs.rs](https://docs.rs/)                                                              |
| Axum Examples | [github.com/tokio-rs/axum/examples](https://github.com/tokio-rs/axum/tree/main/examples) |
