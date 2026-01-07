# Dotenvy

**Environment Variable Loader** - โหลด Environment Variables จากไฟล์ .env

## 📦 ข้อมูลพื้นฐาน

| รายการ     | ข้อมูล                                                         |
| ---------- | -------------------------------------------------------------- |
| Repository | [github.com/allan2/dotenvy](https://github.com/allan2/dotenvy) |
| เอกสาร     | [docs.rs/dotenvy](https://docs.rs/dotenvy)                     |

## 🎯 Dotenvy คืออะไร?

Dotenvy (successor ของ dotenv) ช่วย:

- **Load .env file** - อ่านไฟล์ .env
- **Set environment variables** - ตั้งค่า env vars
- **Development convenience** - แยก config ระหว่าง dev/prod

## 📝 การใช้งานพื้นฐาน

### 1. เพิ่มใน Cargo.toml

```toml
[dependencies]
dotenvy = "0.15"
```

### 2. Basic Usage

```rust
use dotenvy::dotenv;
use std::env;

fn main() {
    // โหลด .env file
    dotenv().ok();

    // อ่าน environment variable
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    println!("Database: {}", database_url);
}
```

### 3. .env File Syntax

```env
# Comments start with #
DATABASE_URL=postgres://user:password@localhost/dbname

# Quotes are optional
API_KEY=my-secret-key
API_KEY="my-secret-key"
API_KEY='my-secret-key'

# Multiline (with quotes)
PRIVATE_KEY="-----BEGIN RSA PRIVATE KEY-----
...
-----END RSA PRIVATE KEY-----"

# Variable expansion
BASE_URL=http://localhost
API_URL=${BASE_URL}/api

# Export keyword (optional)
export DEBUG=true
```

### 4. Error Handling

```rust
use dotenvy::dotenv;

fn main() {
    match dotenv() {
        Ok(path) => println!("Loaded .env from: {:?}", path),
        Err(e) => println!("Warning: {}", e),
    }
}
```

### 5. Custom .env Path

```rust
use dotenvy::from_path;
use std::path::Path;

fn main() {
    from_path(Path::new("custom.env")).ok();
    // หรือ
    from_path(Path::new("/etc/myapp/.env")).ok();
}
```

### 6. Load from String

```rust
use dotenvy::from_read;
use std::io::Cursor;

fn main() {
    let env_content = "DATABASE_URL=sqlite:memory:";
    from_read(Cursor::new(env_content)).ok();
}
```

### 7. Get All Variables

```rust
use dotenvy::dotenv_iter;

fn main() {
    for item in dotenv_iter().unwrap() {
        let (key, value) = item.unwrap();
        println!("{}={}", key, value);
    }
}
```

### 8. Override Existing Variables

```rust
use dotenvy::from_path_override;
use std::path::Path;

fn main() {
    // ปกติจะไม่ override ถ้ามีอยู่แล้ว
    // ใช้ _override เพื่อบังคับ override
    from_path_override(Path::new(".env")).ok();
}
```

## 📚 Functions

| Function            | คำอธิบาย                              |
| ------------------- | ------------------------------------- |
| `dotenv()`          | โหลด .env ใน current/parent directory |
| `from_path(path)`   | โหลดจาก path เฉพาะ                    |
| `from_read(reader)` | โหลดจาก Reader                        |
| `dotenv_override()` | โหลดและ override existing             |
| `dotenv_iter()`     | Iterator ของ key-value pairs          |

## 🔒 Best Practices

### 1. .gitignore

```gitignore
# Never commit .env
.env
.env.local
.env.*.local
```

### 2. สร้าง .env.example

```env
# .env.example - commit ได้ (ไม่มี secrets)
DATABASE_URL=postgres://user:password@localhost/dbname
JWT_SECRET=change-this-in-production
HOST=0.0.0.0
PORT=8080
LOG_LEVEL=info
```

### 3. ใช้ .ok() แทน expect()

```rust
// ✅ Safe - ไม่ panic ถ้าไม่มี .env
dotenv().ok();

// ❌ Dangerous - panic ถ้าไม่มี .env
dotenv().expect("Failed to load .env");
```

### 4. Validate Required Variables

```rust
fn load_config() -> Config {
    dotenv().ok();

    Config {
        database_url: env::var("DATABASE_URL")
            .expect("DATABASE_URL is required"),
        port: env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .expect("PORT must be a number"),
    }
}
```

### 5. Different Files for Environments

```
.env                # Default
.env.development    # Development
.env.production     # Production
.env.test           # Testing
```

```rust
let env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
from_path(Path::new(&format!(".env.{}", env))).ok();
dotenv().ok();  // Fallback to .env
```

## 🔗 Ecosystem

- **config** - Configuration management
- **figment** - Configuration layering
- **envy** - Type-safe env parsing
