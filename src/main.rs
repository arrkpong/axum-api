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
    email: Option<String>,
    phone: Option<String>,
}

/// Payload for token refresh requests
#[derive(Deserialize)]
struct RefreshPayload {
    refresh_token: String,
}

// ============================================================================
// JWT CLAIMS & AUTH
// ============================================================================

/// JWT token claims structure
/// - `sub`: Subject (username)
/// - `user_id`: Database user ID
/// - `jti`: JWT ID (unique token identifier for blacklisting)
/// - `iat`: Issued at timestamp
/// - `exp`: Expiration timestamp
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    user_id: i32,
    jti: String,
    exp: usize,
    iat: usize,
}

/// Authenticated user extracted from JWT token
/// Used as an extractor in protected route handlers
struct AuthUser {
    username: String,
    user_id: i32,
    jti: String,
    exp: chrono::DateTime<Utc>,
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
// HELPER FUNCTIONS
// ============================================================================

/// Generate a new access token for the given user
/// Returns (access_token, claims) or error response
fn generate_access_token(
    username: &str,
    user_id: i32,
    jwt_secret: &str,
) -> Result<(String, Claims), (StatusCode, String)> {
    let now = Utc::now();
    let access_exp = now + Duration::minutes(15);
    let claims = Claims {
        sub: username.to_string(),
        user_id,
        jti: uuid::Uuid::new_v4().to_string(),
        iat: now.timestamp() as usize,
        exp: access_exp.timestamp() as usize,
    };

    match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    ) {
        Ok(token) => Ok((token, claims)),
        Err(e) => {
            error!(%e, "Failed to create access token");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Token creation failed".to_string(),
            ))
        }
    }
}

/// Hash a password using Argon2
fn hash_password(password: &str) -> Result<String, (StatusCode, String)> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(hash) => Ok(hash.to_string()),
        Err(e) => {
            error!(%e, "Password hashing failed");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to hash password".to_string(),
            ))
        }
    }
}

/// Verify a password against a stored hash
fn verify_password(password: &str, password_hash: &str) -> Result<bool, (StatusCode, String)> {
    let parsed_hash = match PasswordHash::new(password_hash) {
        Ok(hash) => hash,
        Err(e) => {
            error!(%e, "Failed to parse stored password hash");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to verify password".to_string(),
            ));
        }
    };

    let argon2 = Argon2::default();
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Generate and store a new refresh token
async fn create_refresh_token(
    user_id: i32,
    db_pool: &PgPool,
) -> Result<String, (StatusCode, String)> {
    let now = Utc::now();
    let refresh_token = uuid::Uuid::new_v4().to_string();
    let refresh_exp = now + Duration::days(7);

    // Store refresh token in DB
    let store_result = sqlx::query(
        "INSERT INTO auth_refresh_tokens (user_id, token, expires_at) VALUES ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(&refresh_token)
    .bind(refresh_exp)
    .execute(db_pool)
    .await;

    match store_result {
        Ok(_) => Ok(refresh_token),
        Err(e) => {
            error!(%e, "Failed to store refresh token");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Login failed".to_string(),
            ))
        }
    }
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
            // Verify password using helper
            if match verify_password(&payload.password, &user.password) {
                Ok(valid) => valid,
                Err((status, msg)) => {
                    return (
                        status,
                        Json(serde_json::json!({"status": "error", "message": msg})),
                    )
                        .into_response();
                }
            } {
                // Generate access token using helper
                let (access_token, _claims) =
                    match generate_access_token(&user.username, user.id, &state.jwt_secret) {
                        Ok(result) => result,
                        Err((status, msg)) => {
                            return (status, msg).into_response();
                        }
                    };

                // Generate refresh token using helper
                let refresh_token = match create_refresh_token(user.id, &state.db_pool).await {
                    Ok(token) => token,
                    Err((status, msg)) => {
                        return (
                            status,
                            Json(serde_json::json!({"status": "error", "message": msg})),
                        )
                            .into_response();
                    }
                };

                info!(user_id = user.id, "Login successful");
                (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "status": "success",
                        "message": "Login successful",
                        "access_token": access_token,
                        "refresh_token": refresh_token,
                        "expires_in": 900,
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

    // Generate random salt and hash password using helper
    let password_hash = match hash_password(&payload.password) {
        Ok(hash) => hash,
        Err((status, msg)) => {
            return (
                status,
                Json(serde_json::json!({"status": "error", "message": msg})),
            )
                .into_response();
        }
    };

    // Insert new user into database
    let insert_user = sqlx::query(
        "INSERT INTO auth_users (username, email, phone, password) VALUES ($1, $2, $3, $4)",
    )
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&payload.phone)
    .bind(&password_hash)
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

/// Logout handler - adds token to blacklist and removes refresh tokens
/// Requires valid JWT token to identify which token to revoke
async fn logout(State(state): State<AppState>, auth_user: AuthUser) -> impl IntoResponse {
    info!("Logout endpoint hit for user: {}", auth_user.username);

    // Insert access token JTI into blacklist
    let blacklist_result = sqlx::query(
        "INSERT INTO auth_token_blacklist (token_jti, user_id, expires_at) VALUES ($1, $2, $3) ON CONFLICT (token_jti) DO NOTHING"
    )
    .bind(&auth_user.jti)
    .bind(auth_user.user_id)
    .bind(auth_user.exp)
    .execute(&state.db_pool)
    .await;

    // Delete all refresh tokens for this user
    let delete_result = sqlx::query("DELETE FROM auth_refresh_tokens WHERE user_id = $1")
        .bind(auth_user.user_id)
        .execute(&state.db_pool)
        .await;

    if blacklist_result.is_err() || delete_result.is_err() {
        error!("Failed to complete logout");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"status": "error", "message": "Logout failed"})),
        )
            .into_response();
    }

    info!("Logout successful - tokens revoked");
    Json(serde_json::json!({
        "status": "success",
        "message": "Logout successful",
    }))
    .into_response()
}

/// Protected profile endpoint
/// Requires valid JWT token (extracted via AuthUser)
async fn profile(auth_user: AuthUser) -> impl IntoResponse {
    info!("Profile endpoint accessed by: {}", auth_user.username);
    Json(serde_json::json!({
        "status": "success",
        "message": format!("Welcome back, {}!", auth_user.username),
        "user_id": auth_user.user_id,
        "username": auth_user.username,
        "email": "TODO", // We will need to fetch this from DB if needed
        "phone": "TODO"
    }))
}

/// Refresh token handler - exchange refresh token for new access token
async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<RefreshPayload>,
) -> impl IntoResponse {
    info!("Refresh token endpoint hit");

    // Find refresh token in DB
    let token_record =
        sqlx::query("SELECT user_id, expires_at FROM auth_refresh_tokens WHERE token = $1")
            .bind(&payload.refresh_token)
            .fetch_optional(&state.db_pool)
            .await;

    let row = match token_record {
        Ok(Some(row)) => row,
        Ok(None) => {
            warn!("Refresh token not found");
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"status": "error", "message": "Invalid refresh token"})),
            )
                .into_response();
        }
        Err(e) => {
            error!(%e, "Database error during refresh");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": "Refresh failed"})),
            )
                .into_response();
        }
    };

    use sqlx::Row;
    let user_id: i32 = row.get("user_id");
    let expires_at: chrono::DateTime<Utc> = row.get("expires_at");

    // Check if refresh token is expired
    if expires_at < Utc::now() {
        // Delete expired token
        let _ = sqlx::query("DELETE FROM auth_refresh_tokens WHERE token = $1")
            .bind(&payload.refresh_token)
            .execute(&state.db_pool)
            .await;
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"status": "error", "message": "Refresh token expired"})),
        )
            .into_response();
    }

    // Get username for claims
    let user_record = sqlx::query("SELECT username FROM auth_users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.db_pool)
        .await;

    let username: String = match user_record {
        Ok(Some(row)) => row.get("username"),
        _ => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": "User not found"})),
            )
                .into_response();
        }
    };

    // Generate new access token using helper
    let (access_token, _claims) = match generate_access_token(&username, user_id, &state.jwt_secret)
    {
        Ok(result) => result,
        Err((status, msg)) => {
            return (status, msg).into_response();
        }
    };

    info!(user_id = user_id, "Token refreshed successfully");
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "success",
            "access_token": access_token,
            "expires_in": 900
        })),
    )
        .into_response()
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

        // Step 4: Check if token is blacklisted
        match token_data {
            Ok(data) => {
                // Check blacklist (using runtime query since table may not exist at compile time)
                let blacklist_check =
                    sqlx::query("SELECT 1 FROM auth_token_blacklist WHERE token_jti = $1")
                        .bind(&data.claims.jti)
                        .fetch_optional(&state.db_pool)
                        .await;

                if blacklist_check.unwrap_or(None).is_some() {
                    return Err((
                        StatusCode::UNAUTHORIZED,
                        Json(
                            serde_json::json!({"status": "error", "message": "Token has been revoked"}),
                        ),
                    ));
                }

                // Convert exp timestamp to DateTime
                let exp_dt = chrono::DateTime::from_timestamp(data.claims.exp as i64, 0)
                    .unwrap_or_else(Utc::now);

                Ok(AuthUser {
                    username: data.claims.sub,
                    user_id: data.claims.user_id,
                    jti: data.claims.jti,
                    exp: exp_dt,
                })
            }
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
        .route("/api/v1/refresh", post(refresh))
        .route("/api/v1/profile", get(profile))
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
