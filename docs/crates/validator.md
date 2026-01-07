# Validator

**Input Validation Library** - ไลบรารีตรวจสอบความถูกต้องของข้อมูล

## 📦 ข้อมูลพื้นฐาน

| รายการ     | ข้อมูล                                                           |
| ---------- | ---------------------------------------------------------------- |
| Repository | [github.com/Keats/validator](https://github.com/Keats/validator) |
| เอกสาร     | [docs.rs/validator](https://docs.rs/validator)                   |

## 🎯 Validator คืออะไร?

Validator ช่วยตรวจสอบ input ก่อนประมวลผล:

- **Declarative** - ใช้ attributes กำหนด rules
- **Customizable** - สร้าง validation function เอง
- **Error messages** - ข้อความ error ที่ชัดเจน

## 🔧 Features ที่มี

| Feature  | คำอธิบาย                      |
| -------- | ----------------------------- |
| `derive` | เปิดใช้ `#[derive(Validate)]` |
| `card`   | Credit card validation        |
| `phone`  | Phone number validation       |
| `unic`   | Unicode validation            |

## 📝 การใช้งานพื้นฐาน

### 1. เพิ่มใน Cargo.toml

```toml
[dependencies]
validator = { version = "0.18", features = ["derive"] }
```

### 2. Basic Validation

```rust
use validator::Validate;

#[derive(Validate)]
struct User {
    #[validate(length(min = 3, max = 50))]
    username: String,

    #[validate(email)]
    email: String,

    #[validate(length(min = 8))]
    password: String,
}

fn create_user(user: User) -> Result<(), ValidationErrors> {
    user.validate()?;
    // ... ดำเนินการต่อ
    Ok(())
}
```

### 3. Built-in Validators

```rust
#[derive(Validate)]
struct Example {
    // Email format
    #[validate(email)]
    email: String,

    // URL format
    #[validate(url)]
    website: String,

    // String length
    #[validate(length(min = 1, max = 100))]
    name: String,

    // Exact length
    #[validate(length(equal = 10))]
    code: String,

    // Numeric range
    #[validate(range(min = 1, max = 100))]
    age: i32,

    // Regex pattern
    #[validate(regex(path = "RE_PHONE"))]
    phone: String,

    // Required (for Option)
    #[validate(required)]
    required_field: Option<String>,

    // Contains substring
    #[validate(contains = "@")]
    must_have_at: String,

    // Does not contain
    #[validate(does_not_contain = "admin")]
    username: String,
}

lazy_static! {
    static ref RE_PHONE: Regex = Regex::new(r"^\d{10}$").unwrap();
}
```

### 4. Nested Validation

```rust
#[derive(Validate)]
struct Address {
    #[validate(length(min = 1))]
    street: String,
    #[validate(length(min = 1))]
    city: String,
}

#[derive(Validate)]
struct User {
    #[validate(length(min = 1))]
    name: String,

    #[validate(nested)]  // ตรวจสอบ Address ด้วย
    address: Address,
}
```

### 5. Custom Error Messages

```rust
#[derive(Validate)]
struct User {
    #[validate(length(min = 3, message = "Username must be at least 3 characters"))]
    username: String,

    #[validate(email(message = "Please enter a valid email address"))]
    email: String,
}
```

### 6. Custom Validation Function

```rust
use validator::ValidationError;

fn validate_username(username: &str) -> Result<(), ValidationError> {
    if username.contains("admin") {
        return Err(ValidationError::new("username_reserved"));
    }
    Ok(())
}

#[derive(Validate)]
struct User {
    #[validate(custom(function = "validate_username"))]
    username: String,
}
```

### 7. Async Validation

```rust
use validator::ValidationError;

async fn validate_email_unique(email: &str) -> Result<(), ValidationError> {
    // ตรวจสอบกับ database
    let exists = check_email_exists(email).await;
    if exists {
        return Err(ValidationError::new("email_taken"));
    }
    Ok(())
}
```

### 8. Validate Vec/HashMap

```rust
#[derive(Validate)]
struct Order {
    #[validate(length(min = 1, message = "At least one item required"))]
    items: Vec<OrderItem>,
}

#[derive(Validate)]
struct OrderItem {
    #[validate(length(min = 1))]
    product_id: String,

    #[validate(range(min = 1))]
    quantity: i32,
}
```

### 9. Error Handling

```rust
use validator::{Validate, ValidationErrors};

fn validate_user(user: &User) {
    match user.validate() {
        Ok(_) => println!("Valid!"),
        Err(errors) => {
            // Get all errors
            for (field, errors) in errors.field_errors() {
                for error in errors {
                    println!("{}: {:?}", field, error.message);
                }
            }
        }
    }
}
```

### 10. Integration with Axum

```rust
use axum::{Json, http::StatusCode};
use validator::Validate;

#[derive(Deserialize, Validate)]
struct CreateUser {
    #[validate(length(min = 3))]
    username: String,
    #[validate(email)]
    email: String,
}

async fn create_user(
    Json(payload): Json<CreateUser>,
) -> Result<Json<User>, (StatusCode, String)> {
    // Validate input
    payload.validate().map_err(|e| {
        (StatusCode::BAD_REQUEST, format!("Validation error: {}", e))
    })?;

    // ... create user
    Ok(Json(user))
}
```

## 📚 Built-in Validators

| Validator                 | หน้าที่                               |
| ------------------------- | ------------------------------------- |
| `email`                   | ตรวจสอบ email format                  |
| `url`                     | ตรวจสอบ URL format                    |
| `length(min, max, equal)` | ความยาว string/vec                    |
| `range(min, max)`         | ช่วงตัวเลข                            |
| `regex`                   | Regular expression                    |
| `required`                | ต้องมีค่า (สำหรับ Option)             |
| `contains`                | ต้องมี substring                      |
| `does_not_contain`        | ต้องไม่มี substring                   |
| `custom`                  | Custom function                       |
| `nested`                  | Validate nested struct                |
| `credit_card`             | Credit card number (ต้องเปิด feature) |
| `phone`                   | Phone number (ต้องเปิด feature)       |

## 🔗 Ecosystem

- **axum** - Web framework integration
- **serde** - Deserialize before validate
- **thiserror** - Custom error types
