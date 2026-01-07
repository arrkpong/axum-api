# Security Crates

**Authentication & Security Libraries** - ไลบรารีสำหรับความปลอดภัย

---

## Argon2

**Password Hashing Algorithm** - Algorithm สำหรับ Hash รหัสผ่าน

### 📦 ข้อมูลพื้นฐาน

| รายการ     | ข้อมูล                                                                                 |
| ---------- | -------------------------------------------------------------------------------------- |
| Repository | [github.com/RustCrypto/password-hashes](https://github.com/RustCrypto/password-hashes) |
| เอกสาร     | [docs.rs/argon2](https://docs.rs/argon2)                                               |

### 🎯 Argon2 คืออะไร?

Argon2 เป็น Password Hashing Algorithm ที่:

- **Winner of PHC** - ชนะ Password Hashing Competition 2015
- **Memory-hard** - ใช้ RAM มากเพื่อป้องกัน GPU/ASIC cracking
- **Recommended by OWASP** - มาตรฐานความปลอดภัย

### 📝 การใช้งาน

```toml
[dependencies]
argon2 = "0.5"
```

#### Hash Password

```rust
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};

fn hash_password(password: &str) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}
```

#### Verify Password

```rust
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordVerifier},
};

fn verify_password(password: &str, hash: &str) -> bool {
    let parsed_hash = PasswordHash::new(hash).unwrap();
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}
```

#### Custom Parameters

```rust
use argon2::{Argon2, Params, Version, Algorithm};

let params = Params::new(
    65536,   // memory cost (KB)
    3,       // time cost (iterations)
    4,       // parallelism
    None,    // output length
)?;

let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
```

### ⚠️ Performance Note

Argon2 เป็น CPU-intensive ควรใช้ `spawn_blocking`:

```rust
use tokio::task::spawn_blocking;

async fn hash_password_async(password: String) -> Result<String, Error> {
    spawn_blocking(move || hash_password(&password)).await?
}
```

---

## JSON Web Token (jsonwebtoken)

**JWT Library** - ไลบรารีสำหรับ JWT

### 📦 ข้อมูลพื้นฐาน

| รายการ     | ข้อมูล                                                                 |
| ---------- | ---------------------------------------------------------------------- |
| Repository | [github.com/Keats/jsonwebtoken](https://github.com/Keats/jsonwebtoken) |
| เอกสาร     | [docs.rs/jsonwebtoken](https://docs.rs/jsonwebtoken)                   |

### 🎯 JWT คืออะไร?

JWT เป็น Token format สำหรับ:

- **Authentication** - ยืนยันตัวตน
- **Authorization** - ตรวจสอบสิทธิ์
- **Stateless** - ไม่ต้องเก็บ session บน server

### 📝 การใช้งาน

```toml
[dependencies]
jsonwebtoken = "10"
```

#### Define Claims

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,    // Subject (user id)
    exp: usize,     // Expiration time (Unix timestamp)
    iat: usize,     // Issued at
    // Custom claims
    role: String,
}
```

#### Create Token (Encode)

```rust
use jsonwebtoken::{encode, Header, EncodingKey};
use std::time::{SystemTime, UNIX_EPOCH};

fn create_token(user_id: &str, secret: &str) -> Result<String, Error> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        exp: now + 3600,  // หมดอายุใน 1 ชั่วโมง
        iat: now,
        role: "user".to_string(),
    };

    let token = encode(
        &Header::default(),  // HS256
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(token)
}
```

#### Verify Token (Decode)

```rust
use jsonwebtoken::{decode, Validation, DecodingKey};

fn verify_token(token: &str, secret: &str) -> Result<Claims, Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}
```

#### Custom Validation

```rust
use jsonwebtoken::{Validation, Algorithm};

let mut validation = Validation::new(Algorithm::HS256);
validation.set_issuer(&["my-app"]);
validation.set_audience(&["my-api"]);
validation.validate_exp = true;  // ตรวจสอบ expiration
validation.leeway = 60;  // อนุญาต clock skew 60 วินาที

let token_data = decode::<Claims>(&token, &key, &validation)?;
```

#### Different Algorithms

```rust
use jsonwebtoken::{Header, Algorithm, EncodingKey};

// HS256 (Symmetric - ใช้ secret เดียวกัน)
let token = encode(&Header::new(Algorithm::HS256), &claims, &EncodingKey::from_secret(secret))?;

// RS256 (Asymmetric - ใช้ private/public key)
let token = encode(&Header::new(Algorithm::RS256), &claims, &EncodingKey::from_rsa_pem(private_key)?)?;

// ES256 (Asymmetric - ECDSA)
let token = encode(&Header::new(Algorithm::ES256), &claims, &EncodingKey::from_ec_pem(private_key)?)?;
```

### 📚 Standard Claims

| Claim | ชื่อเต็ม        | หน้าที่                |
| ----- | --------------- | ---------------------- |
| `sub` | Subject         | ระบุตัวตน (user id)    |
| `exp` | Expiration Time | เวลาหมดอายุ            |
| `iat` | Issued At       | เวลาที่สร้าง           |
| `nbf` | Not Before      | ใช้ได้ตั้งแต่เมื่อไหร่ |
| `iss` | Issuer          | ผู้ออก token           |
| `aud` | Audience        | ใครเป็นผู้รับ          |
| `jti` | JWT ID          | ID เฉพาะของ token      |

### 🔧 Features

| Feature       | คำอธิบาย                   |
| ------------- | -------------------------- |
| `use_pem`     | รองรับ PEM format          |
| `rust_crypto` | ใช้ RustCrypto (pure Rust) |

---

## 🔗 Security Flow

```
1. User Login
   ↓
2. Verify Password (Argon2)
   ↓
3. Create JWT (jsonwebtoken)
   ↓
4. Send Token to Client
   ↓
5. Client sends Token in Header
   ↓
6. Verify JWT (jsonwebtoken)
   ↓
7. Allow/Deny Access
```
