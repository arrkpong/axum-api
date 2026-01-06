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

use tower_http::trace::TraceLayer;
use tracing::{Level, error, info, instrument, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

async fn index() -> impl IntoResponse {
    info!("Index endpoint hit");
    Json(serde_json::json!({
        "status": "success",
        "message": "Welcome to the Index API",
    }))
}

#[instrument(skip(pool, payload), fields(username = %payload.username))]
async fn login(State(pool): State<PgPool>, Json(payload): Json<LoginPayload>) -> impl IntoResponse {
    info!("Login endpoint hit");

    let select_user = sqlx::query!(
        "SELECT id, username, password FROM auth_users WHERE username = $1",
        &payload.username,
    )
    .fetch_optional(&pool)
    .await;

    match select_user {
        Ok(Some(user)) => {
            if user.password == payload.password {
                info!(user_id = user.id, "Login successful");
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
                warn!("Login failed: Incorrect password");
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({"status": "error", "message": "Invalid credentials"})),
                )
                    .into_response()
            }
        }

        Ok(None) => {
            warn!("Login failed: User not found");
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"status": "error", "message": "Invalid credentials"})),
            )
                .into_response()
        }

        Err(e) => {
            error!(%e, "Database query failed during login");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": "Failed to fetch user"})),
            )
                .into_response()
        }
    }
}

#[instrument(skip(pool, payload), fields(username = %payload.username))]
async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
    info!("Register endpoint hit");

    let insert_user = sqlx::query!(
        "INSERT INTO auth_users (username, password) VALUES ($1, $2)",
        &payload.username,
        &payload.password,
    )
    .execute(&pool)
    .await;

    match insert_user {
        Ok(_) => {
            info!("User registered successfully");
            (
                StatusCode::CREATED,
                Json(
                    serde_json::json!({"status": "success", "message": "User registered successfully"}),
                ),
            ).into_response()
        }
        Err(e) => {
            if let Some(db_error) = e.as_database_error()
                && db_error.is_unique_violation()
            {
                warn!(%db_error, "Registration failed: Unique constraint violation");

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

            error!(%e, "Database insertion failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": "Failed to register user"})),
            )
                .into_response()
        }
    }
}

async fn logout() -> impl IntoResponse {
    info!("Logout endpoint hit");
    Json(serde_json::json!({
        "status": "success",
        "message": "Logout successful",
    }))
}

async fn admin() -> impl IntoResponse {
    info!("Admin endpoint hit");
    Json(serde_json::json!({
        "status": "success",
        "message": "Welcome to the admin panel",
    }))
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .with(tracing_subscriber::filter::LevelFilter::from_level(
            Level::INFO,
        ))
        .init();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    info!("⏳ Connecting to Database...");

    let db_pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(std::time::Duration::from_secs(3))
        .connect(&db_url)
        .await
        .expect("❌ Failed to connect to Postgres");

    info!("✅ Database connected successfully!");

    let app = Router::new()
        .route("/", get(index))
        .route("/api/v1/login", post(login))
        .route("/api/v1/register", post(register))
        .route("/api/v1/logout", post(logout))
        .route("/api/v1/admin", get(admin))
        .with_state(db_pool)
        .layer(TraceLayer::new_for_http());

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("{}:{}", host, port);

    info!("🚀 Server is starting on {}", addr);
    info!("👉 Open in browser: http://localhost:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
