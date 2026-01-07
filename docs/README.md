# Documentation

เอกสารสำหรับโปรเจค Axum API

## 📚 สารบัญ

### [Crates Reference](./crates/README.md)

เอกสารอธิบายการใช้งาน Rust Crates พื้นฐาน (ภาษาไทย)

| หมวด          | Crates                |
| ------------- | --------------------- |
| Web Framework | axum, tokio           |
| Middleware    | tower, tower-http     |
| Database      | sqlx                  |
| Serialization | serde                 |
| Security      | argon2, jsonwebtoken  |
| Utilities     | chrono, uuid, dotenvy |
| Validation    | validator, thiserror  |
| Logging       | tracing               |
| API Docs      | utoipa                |

---

## 📁 โครงสร้าง

```
docs/
├── README.md           ← คุณอยู่ที่นี่
└── crates/             ← Rust Crates Reference
    ├── README.md       ← Index ของ crates ทั้งหมด
    ├── axum.md
    ├── tokio.md
    └── ...
```

---

## 🔗 Quick Links

- [Axum (Web Framework)](./crates/axum.md)
- [SQLx (Database)](./crates/sqlx.md)
- [Security (Argon2 + JWT)](./crates/security.md)
- [Tracing (Logging)](./crates/tracing.md)
