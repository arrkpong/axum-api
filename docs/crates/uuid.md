# UUID

**Universally Unique Identifier** - สร้าง ID ที่ไม่ซ้ำกัน

## 📦 ข้อมูลพื้นฐาน

| รายการ     | ข้อมูล                                                     |
| ---------- | ---------------------------------------------------------- |
| Repository | [github.com/uuid-rs/uuid](https://github.com/uuid-rs/uuid) |
| เอกสาร     | [docs.rs/uuid](https://docs.rs/uuid)                       |

## 🎯 UUID คืออะไร?

UUID (Universally Unique Identifier) คือ:

- **128-bit identifier** - ขนาด 16 bytes
- **Globally unique** - ไม่ซ้ำกันทั่วโลก
- **No coordination** - ไม่ต้องมี central server

## 🔧 Features ที่มี

| Feature | คำอธิบาย                 |
| ------- | ------------------------ |
| `v4`    | Random UUID (แนะนำ)      |
| `v1`    | Timestamp + MAC address  |
| `v3`    | Name-based (MD5)         |
| `v5`    | Name-based (SHA-1)       |
| `v7`    | Timestamp-ordered (ใหม่) |
| `serde` | Serialize/Deserialize    |
| `js`    | JavaScript/WebAssembly   |

## 📝 การใช้งานพื้นฐาน

### 1. เพิ่มใน Cargo.toml

```toml
[dependencies]
uuid = { version = "1", features = ["v4", "serde"] }
```

### 2. Generate UUID v4 (Random)

```rust
use uuid::Uuid;

let id = Uuid::new_v4();
println!("{}", id);
// 550e8400-e29b-41d4-a716-446655440000
```

### 3. UUID Formats

```rust
let id = Uuid::new_v4();

// Standard hyphenated
println!("{}", id);
// 550e8400-e29b-41d4-a716-446655440000

// Hyphenated lowercase
println!("{}", id.hyphenated());
// 550e8400-e29b-41d4-a716-446655440000

// Simple (no hyphens)
println!("{}", id.simple());
// 550e8400e29b41d4a716446655440000

// URN format
println!("{}", id.urn());
// urn:uuid:550e8400-e29b-41d4-a716-446655440000

// Braced
println!("{}", id.braced());
// {550e8400-e29b-41d4-a716-446655440000}
```

### 4. Parse UUID

```rust
use uuid::Uuid;

// From string
let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")?;

// From string (alternative)
let id: Uuid = "550e8400-e29b-41d4-a716-446655440000".parse()?;

// From bytes
let bytes = [0x55, 0x0e, 0x84, 0x00, 0xe2, 0x9b, 0x41, 0xd4,
             0xa7, 0x16, 0x44, 0x66, 0x55, 0x44, 0x00, 0x00];
let id = Uuid::from_bytes(bytes);
```

### 5. UUID v1 (Timestamp + MAC)

```rust
use uuid::{Uuid, Timestamp, NoContext};

let ts = Timestamp::now(NoContext);
let id = Uuid::new_v1(ts, &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06]);
```

### 6. UUID v5 (Name-based SHA-1)

```rust
use uuid::{Uuid, uuid};

// ใช้ namespace + name
let namespace = uuid!("6ba7b810-9dad-11d1-80b4-00c04fd430c8"); // URL namespace
let id = Uuid::new_v5(&namespace, b"https://example.com");
```

### 7. UUID v7 (Timestamp-ordered)

```rust
use uuid::Uuid;

// Sortable UUID
let id = Uuid::now_v7();
```

### 8. Check Version

```rust
use uuid::Uuid;

let id = Uuid::new_v4();

println!("Version: {:?}", id.get_version());
// Version: Some(Random)

println!("Is nil: {}", id.is_nil());
// Is nil: false
```

### 9. Nil UUID

```rust
use uuid::Uuid;

// สร้าง nil UUID (all zeros)
let nil = Uuid::nil();
// 00000000-0000-0000-0000-000000000000

// ตรวจสอบ
if id.is_nil() {
    println!("ID is nil");
}
```

### 10. Max UUID

```rust
use uuid::Uuid;

// Max UUID (all ones)
let max = Uuid::max();
// ffffffff-ffff-ffff-ffff-ffffffffffff

if id.is_max() {
    println!("ID is max");
}
```

### 11. Convert to Bytes

```rust
let id = Uuid::new_v4();

// As bytes array
let bytes: [u8; 16] = *id.as_bytes();

// As u128
let n: u128 = id.as_u128();
```

### 12. Use with Serde

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct User {
    id: Uuid,
    name: String,
}

let user = User {
    id: Uuid::new_v4(),
    name: "สมชาย".to_string(),
};

let json = serde_json::to_string(&user)?;
// {"id":"550e8400-e29b-41d4-a716-446655440000","name":"สมชาย"}
```

## 📚 UUID Versions

| Version | วิธีสร้าง       | Use Case                   |
| ------- | --------------- | -------------------------- |
| v1      | Timestamp + MAC | Sequential, traceable      |
| v3      | Name + MD5      | Deterministic (deprecated) |
| v4      | Random          | General purpose (แนะนำ)    |
| v5      | Name + SHA-1    | Deterministic              |
| v7      | Timestamp       | Sortable, modern (แนะนำ)   |

## 🔗 Ecosystem

- **serde** - JSON serialization
- **sqlx** - Database UUID type
- **diesel** - Database UUID type
