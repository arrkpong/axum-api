# Utoipa

**OpenAPI Documentation Generator** - สร้าง API Documentation อัตโนมัติ

## 📦 ข้อมูลพื้นฐาน

| รายการ     | ข้อมูล                                                       |
| ---------- | ------------------------------------------------------------ |
| Repository | [github.com/juhaku/utoipa](https://github.com/juhaku/utoipa) |
| เอกสาร     | [docs.rs/utoipa](https://docs.rs/utoipa)                     |

## 🎯 Utoipa คืออะไร?

Utoipa ช่วยสร้าง OpenAPI (Swagger) documentation:

- **Auto-generate** - สร้าง spec จาก Rust code
- **Type-safe** - ดึง info จาก Rust types
- **Swagger UI** - แสดง interactive docs

## 🔧 Features ที่มี

| Feature         | คำอธิบาย              |
| --------------- | --------------------- |
| `actix_extras`  | Actix-web integration |
| `axum_extras`   | Axum integration      |
| `rocket_extras` | Rocket integration    |
| `uuid`          | UUID type support     |
| `chrono`        | DateTime support      |
| `time`          | time crate support    |
| `decimal`       | rust_decimal support  |
| `url`           | URL type support      |

## 📝 การใช้งานพื้นฐาน

### 1. เพิ่มใน Cargo.toml

```toml
[dependencies]
utoipa = { version = "5", features = ["axum_extras"] }
utoipa-swagger-ui = { version = "9", features = ["axum"] }
```

### 2. Define Schema

```rust
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
struct User {
    /// User's unique identifier
    id: i64,
    /// User's display name
    #[schema(example = "John Doe")]
    name: String,
    /// User's email address
    #[schema(example = "john@example.com")]
    email: Option<String>,
}
```

### 3. Document Handler

```rust
use utoipa::path;

/// Get user by ID
#[utoipa::path(
    get,
    path = "/users/{id}",
    params(
        ("id" = i64, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User found", body = User),
        (status = 404, description = "User not found"),
    ),
    tag = "users"
)]
async fn get_user(Path(id): Path<i64>) -> impl IntoResponse {
    // ...
}
```

### 4. Create OpenAPI Spec

```rust
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(get_user, create_user, list_users),
    components(schemas(User, CreateUser)),
    tags(
        (name = "users", description = "User management endpoints")
    ),
    info(
        title = "My API",
        version = "1.0.0",
        description = "API documentation"
    )
)]
struct ApiDoc;
```

### 5. Setup Swagger UI (Axum)

```rust
use utoipa_swagger_ui::SwaggerUi;

let app = Router::new()
    .route("/users/:id", get(get_user))
    .merge(SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", ApiDoc::openapi()));
```

### 6. Request Body

```rust
#[derive(Deserialize, ToSchema)]
struct CreateUser {
    #[schema(example = "johndoe")]
    username: String,
    #[schema(example = "john@example.com")]
    email: String,
    #[schema(example = "password123", min_length = 8)]
    password: String,
}

#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUser,
    responses(
        (status = 201, description = "User created", body = User),
        (status = 400, description = "Invalid input"),
    )
)]
async fn create_user(Json(payload): Json<CreateUser>) -> impl IntoResponse {
    // ...
}
```

### 7. Query Parameters

```rust
#[derive(Deserialize, IntoParams)]
struct Pagination {
    #[param(example = 1)]
    page: Option<u32>,
    #[param(example = 10, maximum = 100)]
    limit: Option<u32>,
}

#[utoipa::path(
    get,
    path = "/users",
    params(Pagination),
    responses(
        (status = 200, body = Vec<User>),
    )
)]
async fn list_users(Query(params): Query<Pagination>) -> impl IntoResponse {
    // ...
}
```

### 8. Headers

```rust
#[utoipa::path(
    get,
    path = "/protected",
    params(
        ("Authorization" = String, Header, description = "Bearer token")
    ),
    responses(
        (status = 200, description = "Success"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
async fn protected_endpoint() -> impl IntoResponse {
    // ...
}
```

### 9. Security Schemes

```rust
#[derive(OpenApi)]
#[openapi(
    paths(...),
    components(
        schemas(...),
        securitySchemes(
            ("bearer_auth" = (
                ty = "http",
                scheme = "bearer",
                bearer_format = "JWT"
            ))
        )
    ),
    security(("bearer_auth" = []))
)]
struct ApiDoc;
```

### 10. Enums

```rust
#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
enum Status {
    Active,
    Inactive,
    Pending,
}

#[derive(Serialize, Deserialize, ToSchema)]
struct UserWithStatus {
    id: i64,
    status: Status,
}
```

### 11. Nested Objects

```rust
#[derive(Serialize, ToSchema)]
struct Address {
    street: String,
    city: String,
    country: String,
}

#[derive(Serialize, ToSchema)]
struct UserProfile {
    user: User,
    address: Address,
}
```

### 12. Generic Types

```rust
#[derive(Serialize, ToSchema)]
#[aliases(UserPage = Page<User>)]
struct Page<T: ToSchema> {
    items: Vec<T>,
    total: i64,
    page: i32,
}
```

## 📚 Schema Attributes

| Attribute                    | หน้าที่                   |
| ---------------------------- | ------------------------- |
| `#[schema(example = "...")]` | ตัวอย่างค่า               |
| `#[schema(min_length = N)]`  | ความยาวขั้นต่ำ            |
| `#[schema(max_length = N)]`  | ความยาวสูงสุด             |
| `#[schema(minimum = N)]`     | ค่าต่ำสุด                 |
| `#[schema(maximum = N)]`     | ค่าสูงสุด                 |
| `#[schema(format = "...")]`  | Format (email, uri, etc.) |
| `#[schema(nullable)]`        | อาจเป็น null              |
| `#[schema(deprecated)]`      | Deprecated field          |

## 📚 Path Attributes

| Attribute      | หน้าที่                       |
| -------------- | ----------------------------- |
| `method`       | HTTP method (get, post, etc.) |
| `path`         | URL path                      |
| `params`       | Path/Query/Header parameters  |
| `request_body` | Request body type             |
| `responses`    | Response definitions          |
| `tag`          | Grouping tag                  |
| `security`     | Security requirements         |

## 🔗 Ecosystem

- **utoipa-swagger-ui** - Swagger UI
- **utoipa-redoc** - ReDoc UI
- **utoipa-rapidoc** - RapiDoc UI
