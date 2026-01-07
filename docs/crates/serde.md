# Serde

**Serialization Framework** - เฟรมเวิร์คสำหรับ Serialize/Deserialize

## 📦 ข้อมูลพื้นฐาน

| รายการ     | ข้อมูล                                                         |
| ---------- | -------------------------------------------------------------- |
| Repository | [github.com/serde-rs/serde](https://github.com/serde-rs/serde) |
| เอกสาร     | [docs.rs/serde](https://docs.rs/serde)                         |

## 🎯 Serde คืออะไร?

Serde เป็น Serialization framework ที่ช่วย:

- **Serialize** - แปลง Rust struct → JSON/YAML/TOML
- **Deserialize** - แปลง JSON/YAML/TOML → Rust struct
- **Zero-cost** - ไม่มี runtime overhead

## 🔧 Features ที่มี

| Feature  | คำอธิบาย                                    |
| -------- | ------------------------------------------- |
| `derive` | เปิดใช้ `#[derive(Serialize, Deserialize)]` |
| `std`    | Standard library support (default)          |
| `alloc`  | Alloc-only support                          |
| `rc`     | รองรับ `Rc<T>` และ `Arc<T>`                 |

## 📝 การใช้งานพื้นฐาน

### 1. เพิ่มใน Cargo.toml

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"  # สำหรับ JSON
```

### 2. Basic Derive

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: Option<String>,
}
```

### 3. Serialize (Struct → JSON)

```rust
let user = User {
    id: 1,
    name: "สมชาย".to_string(),
    email: Some("somchai@example.com".to_string()),
};

// Pretty print
let json = serde_json::to_string_pretty(&user)?;
println!("{}", json);
// {
//   "id": 1,
//   "name": "สมชาย",
//   "email": "somchai@example.com"
// }

// Compact
let json = serde_json::to_string(&user)?;
// {"id":1,"name":"สมชาย","email":"somchai@example.com"}
```

### 4. Deserialize (JSON → Struct)

```rust
let json = r#"{"id":1,"name":"สมชาย"}"#;
let user: User = serde_json::from_str(json)?;
```

### 5. Rename Fields

```rust
#[derive(Serialize, Deserialize)]
struct User {
    #[serde(rename = "userId")]
    id: u64,

    #[serde(rename = "fullName")]
    name: String,
}
// JSON: {"userId":1,"fullName":"สมชาย"}
```

### 6. Rename All Fields

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct User {
    user_id: u64,      // → "userId"
    full_name: String, // → "fullName"
}

// Options: camelCase, snake_case, PascalCase, SCREAMING_SNAKE_CASE, kebab-case
```

### 7. Skip Fields

```rust
#[derive(Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,

    #[serde(skip)]  // ไม่ serialize และ deserialize
    internal_state: String,

    #[serde(skip_serializing)]  // serialize ไม่ส่งออก
    password: String,

    #[serde(skip_deserializing, default)]  // deserialize ไม่รับเข้า
    created_at: DateTime<Utc>,
}
```

### 8. Default Values

```rust
#[derive(Serialize, Deserialize)]
struct Pagination {
    #[serde(default = "default_page")]
    page: u32,

    #[serde(default)]  // ใช้ Default trait
    limit: u32,
}

fn default_page() -> u32 { 1 }

impl Default for Pagination {
    fn default() -> Self {
        Self { page: 1, limit: 10 }
    }
}
```

### 9. Flatten Nested Structs

```rust
#[derive(Serialize, Deserialize)]
struct Pagination {
    page: u32,
    limit: u32,
}

#[derive(Serialize, Deserialize)]
struct ListUsers {
    #[serde(flatten)]
    pagination: Pagination,
    sort_by: String,
}
// JSON: {"page":1,"limit":10,"sort_by":"name"}
// ไม่ใช่: {"pagination":{"page":1,"limit":10},"sort_by":"name"}
```

### 10. Enums

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Status {
    Active,
    Inactive,
    Pending,
}
// JSON: "active", "inactive", "pending"

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum Event {
    Click { x: i32, y: i32 },
    KeyPress { key: String },
}
// JSON: {"type":"Click","x":10,"y":20}
```

### 11. Custom Serialization

```rust
use serde::{Serializer, Deserializer};

#[derive(Serialize, Deserialize)]
struct User {
    #[serde(serialize_with = "serialize_uppercase")]
    name: String,
}

fn serialize_uppercase<S>(value: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_uppercase())
}
```

### 12. Deny Unknown Fields

```rust
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    host: String,
    port: u16,
}
// Error ถ้า JSON มี field ที่ไม่รู้จัก
```

## 📚 Attributes สรุป

| Attribute                       | หน้าที่                    |
| ------------------------------- | -------------------------- |
| `#[serde(rename = "...")]`      | เปลี่ยนชื่อ field          |
| `#[serde(rename_all = "...")]`  | เปลี่ยนชื่อทุก fields      |
| `#[serde(skip)]`                | ไม่ serialize/deserialize  |
| `#[serde(default)]`             | ใช้ค่า default             |
| `#[serde(flatten)]`             | รวม nested struct          |
| `#[serde(with = "...")]`        | Custom serializer          |
| `#[serde(deny_unknown_fields)]` | Error ถ้ามี unknown fields |

## 🔗 Data Formats

| Crate        | Format |
| ------------ | ------ |
| `serde_json` | JSON   |
| `serde_yaml` | YAML   |
| `toml`       | TOML   |
| `serde_cbor` | CBOR   |
| `bincode`    | Binary |
