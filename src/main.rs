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

async fn login() -> &'static str {
    "Login endpoint"
}
async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
    let result = sqlx::query!(
        "INSERT INTO auth_users (username, password) VALUES ($1, $2)",
        &payload.username,
        &payload.password,
    )
    .execute(&pool)
    .await;
    match result {
        Ok(_) => (
            StatusCode::CREATED,
            Json(
                serde_json::json!({"status": "success", "message": "User registered successfully"}),
            ),
        ),
        Err(e) => {
            if let Some(db_error) = e.as_database_error()
                && db_error.is_unique_violation()
            {
                return match db_error.constraint() {
                    Some("auth_users_username_key") => (
                        StatusCode::CONFLICT,
                        Json(
                            serde_json::json!({"status": "error", "message": "Username already exists"}),
                        ),
                    ),
                    _ => (
                        StatusCode::CONFLICT,
                        Json(
                            serde_json::json!({"status": "error", "message": "Unique constraint violation"}),
                        ),
                    ),
                };
            }

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": "Failed to register user"})),
            )
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
