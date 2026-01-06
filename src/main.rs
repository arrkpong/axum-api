use axum::response::IntoResponse;
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use tower_http::trace::TraceLayer;
use tracing::{error, info, instrument, warn};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};

// ============================================================================
// REQUEST/RESPONSE PAYLOADS
// ============================================================================

/// Payload for user login requests
#[derive(Deserialize)]
struct LoginPayload {
    username: String,
    password: String,
}

/// Payload for user registration requests
#[derive(Deserialize)]
struct RegisterPayload {
    username: String,
    password: String,
}

// ============================================================================
// JWT CLAIMS & AUTH
// ============================================================================

/// JWT token claims structure
/// - `sub`: Subject (username)
/// - `user_id`: Database user ID
/// - `iat`: Issued at timestamp
/// - `exp`: Expiration timestamp
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    user_id: i32,
    exp: usize,
    iat: usize,
}

/// Authenticated user extracted from JWT token
/// Used as an extractor in protected route handlers
struct AuthUser {
    username: String,
    user_id: i32,
}

// ============================================================================
// APPLICATION STATE
// ============================================================================

/// Shared application state passed to all handlers
/// Contains database connection pool and JWT secret
#[derive(Clone)]
struct AppState {
    db_pool: PgPool,
    jwt_secret: String,
}

// ============================================================================
// ROUTE HANDLERS
// ============================================================================

/// Health check / welcome endpoint
async fn index() -> impl IntoResponse {
    info!("Index endpoint hit");
    Json(serde_json::json!({
        "status": "success",
        "message": "Welcome to the Index API",
    }))
}

/// User login handler
/// - Validates credentials against database
/// - Verifies password using Argon2
/// - Returns JWT token on success
#[instrument(skip(state, payload), fields(username = %payload.username))]
async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> impl IntoResponse {
    info!("Login endpoint hit");

    // Query user from database
    let select_user = sqlx::query!(
        "SELECT id, username, password FROM auth_users WHERE username = $1",
        &payload.username,
    )
    .fetch_optional(&state.db_pool)
    .await;

    match select_user {
        Ok(Some(user)) => {
            // Parse the stored password hash
            let parsed_hash = match PasswordHash::new(&user.password) {
                Ok(hash) => hash,
                Err(e) => {
                    error!(%e, "Failed to parse stored password hash");
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"status": "error", "message": "Failed to verify password"})),
                    )
                        .into_response();
                }
            };
            // Verify password using Argon2
            let argon2 = Argon2::default();
            if argon2
                .verify_password(payload.password.as_bytes(), &parsed_hash)
                .is_ok()
            {
                // Generate JWT token with 24-hour expiration
                let now = Utc::now();
                let exp = now + Duration::hours(24);
                let claims = Claims {
                    sub: user.username.clone(),
                    user_id: user.id,
                    iat: now.timestamp() as usize,
                    exp: exp.timestamp() as usize,
                };
                let token = match encode(
                    &Header::default(),
                    &claims,
                    &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
                ) {
                    Ok(t) => t,
                    Err(e) => {
                        error!(%e, "Failed to create token");
                        return (StatusCode::INTERNAL_SERVER_ERROR, "Token creation failed")
                            .into_response();
                    }
                };
                info!(user_id = user.id, "Login successful");
                (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "status": "success",
                        "message": "Login successful",
                        "token": token,
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

/// User registration handler
/// - Hashes password using Argon2 with random salt
/// - Stores new user in database
/// - Returns conflict error if username exists
#[instrument(skip(state, payload), fields(username = %payload.username))]
async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
    info!("Register endpoint hit");

    // Generate random salt and hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = match argon2.hash_password(payload.password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(e) => {
            error!(%e, "Password hashing failed");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": "Failed to hash password"})),
            )
                .into_response();
        }
    };

    // Insert new user into database
    let insert_user = sqlx::query!(
        "INSERT INTO auth_users (username, password) VALUES ($1, $2)",
        payload.username,
        password_hash,
    )
    .execute(&state.db_pool)
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

/// Logout handler (stateless - just returns success)
async fn logout() -> impl IntoResponse {
    info!("Logout endpoint hit");
    Json(serde_json::json!({
        "status": "success",
        "message": "Logout successful",
    }))
}

/// Protected admin endpoint
/// Requires valid JWT token (extracted via AuthUser)
async fn admin(auth_user: AuthUser) -> impl IntoResponse {
    info!("Admin endpoint accessed by: {}", auth_user.username);
    Json(serde_json::json!({
        "status": "success",
        "message": format!("Welcome back, {}!", auth_user.username),
        "user_id": auth_user.user_id
    }))
}

// ============================================================================
// JWT TOKEN EXTRACTOR
// ============================================================================

/// Custom extractor for authenticated users
/// Validates JWT token from Authorization header and extracts user info
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Step 1: Extract Authorization header
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok());

        let auth_header = match auth_header {
            Some(header) => header,
            None => {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(
                        serde_json::json!({"status": "error", "message": "Missing Authorization header"}),
                    ),
                ));
            }
        };

        // Step 2: Validate "Bearer " prefix
        if !auth_header.starts_with("Bearer ") {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"status": "error", "message": "Invalid header format"})),
            ));
        }

        // Step 3: Extract and verify JWT token
        let token = &auth_header[7..];
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
            &Validation::default(),
        );

        // Step 4: Return authenticated user or error
        match token_data {
            Ok(data) => Ok(AuthUser {
                username: data.claims.sub,
                user_id: data.claims.user_id,
            }),
            Err(_) => Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"status": "error", "message": "Invalid or expired token"})),
            )),
        }
    }
}

// ============================================================================
// APPLICATION ENTRY POINT
// ============================================================================

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv().ok();

    // Initialize tracing subscriber for logging
    // Read LOG_LEVEL from env, default to "info" if not set
    let log_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .with(EnvFilter::new(&log_level))
        .init();

    // Connect to PostgreSQL database
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    info!("⏳ Connecting to Database...");

    let db_pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(std::time::Duration::from_secs(3))
        .connect(&db_url)
        .await
        .expect("❌ Failed to connect to Postgres");

    info!("✅ Database connected successfully!");

    // Load JWT secret and create shared application state
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let app_state = AppState {
        db_pool,
        jwt_secret,
    };

    // Configure routes and middleware
    let app = Router::new()
        .route("/", get(index))
        .route("/api/v1/login", post(login))
        .route("/api/v1/register", post(register))
        .route("/api/v1/logout", post(logout))
        .route("/api/v1/admin", get(admin))
        .with_state(app_state)
        .layer(TraceLayer::new_for_http());

    // Bind to host and port from environment or defaults
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("{}:{}", host, port);

    info!("🚀 Server is starting on {}", addr);
    info!("👉 Open in browser: http://localhost:{}", port);

    // Start the server
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
