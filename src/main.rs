use axum::response::IntoResponse;
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use dotenvy::dotenv;
use serde::Deserialize;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

// use argon2::{
//     Argon2,
//     password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
// };

#[derive(Deserialize)]
struct LoginPayload {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct RegisterPayload {
    username: String,
    password: String,
}

async fn index(State(_db_pool): State<PgPool>) -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "success",
        "message": "Welcome to the API",
    }))
}

async fn login(State(pool): State<PgPool>, Json(payload): Json<LoginPayload>) -> impl IntoResponse {
    let select_user = sqlx::query!(
        "SELECT id, username, password FROM auth_users WHERE username = $1",
        &payload.username,
    )
    .fetch_optional(&pool)
    .await;

    match select_user {
        Ok(Some(user)) => {
            if user.password == payload.password {
                (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "status": "success",
                        "message": "Login successful",
                        "user_id": user.id
                    })),
                )
                    .into_response()
            } else {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({"status": "error", "message": "Invalid credentials"})),
                )
                    .into_response()
            }
        }

        Ok(None) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"status": "error", "message": "Invalid credentials"})),
        )
            .into_response(),

        Err(e) => {
            eprintln!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": "Failed to fetch user"})),
            )
                .into_response()
        }
    }
}
async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
    let insert_user = sqlx::query!(
        "INSERT INTO auth_users (username, password) VALUES ($1, $2)",
        &payload.username,
        &payload.password,
    )
    .execute(&pool)
    .await;

    match insert_user {
        Ok(_) => (
            StatusCode::CREATED,
            Json(
                serde_json::json!({"status": "success", "message": "User registered successfully"}),
            ),
        )
            .into_response(),
        Err(e) => {
            if let Some(db_error) = e.as_database_error()
                && db_error.is_unique_violation()
            {
                return match db_error.constraint() {
                    Some("auth_users_username_key") => (
                        StatusCode::CONFLICT,
                        Json(serde_json::json!({"status": "error", "message": "Username already exists"})),
                    ).into_response(),
                    _ => (
                        StatusCode::CONFLICT,
                        Json(serde_json::json!({"status": "error", "message": "Unique constraint violation"})),
                    ).into_response(),
                };
            }
            eprintln!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": "Failed to register user"})),
            )
                .into_response()
        }
    }
}

async fn logout() -> &'static str {
    "Logout endpoint"
}

async fn admin() -> &'static str {
    "Admin endpoint"
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    println!("⏳ Connecting to Database...");
    let db_pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(std::time::Duration::from_secs(3))
        .connect(&db_url)
        .await
        .expect("❌ Failed to connect to Postgres");
    println!("✅ Database connected successfully!");

    let app = Router::new()
        .route("/", get(index))
        .route("/api/v1/login", post(login))
        .route("/api/v1/register", post(register))
        .route("/api/v1/logout", post(logout))
        .route("/api/v1/admin", get(admin))
        .with_state(db_pool);

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("{}:{}", host, port);
    println!("🚀 Server is starting on {}", addr);
    println!("👉 Open in browser: http://localhost:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
