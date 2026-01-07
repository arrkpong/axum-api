use axum::response::IntoResponse;
use axum::{
    BoxError, Json, Router,
    error_handling::HandleErrorLayer,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

use tower::ServiceBuilder;
use tower::buffer::BufferLayer;
use tower::limit::RateLimitLayer;
use tower::timeout::TimeoutLayer;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{error, info, instrument, warn};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use chrono::{Duration as ChronoDuration, Utc};
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
// CONFIGURATION
// ============================================================================

/// Application configuration loaded from environment variables
#[derive(Clone)]
struct Config {
    /// JWT secret key for signing tokens
    jwt_secret: String,
    /// Access token expiry in minutes
    access_token_expiry_minutes: i64,
    /// Refresh token expiry in days
    refresh_token_expiry_days: i64,
    /// CORS allowed origin ("*" for permissive)
    cors_origin: String,
    /// Rate limit: requests per window
    rate_limit_requests: u64,
    /// Rate limit: window in seconds
    rate_limit_seconds: u64,
    /// Request timeout in seconds
    request_timeout_seconds: u64,
    /// Database pool max connections
    db_pool_max: u32,
}

impl Config {
    /// Load configuration from environment variables
    fn from_env() -> Self {
        Self {
            jwt_secret: std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            access_token_expiry_minutes: std::env::var("ACCESS_TOKEN_EXPIRY_MINUTES")
                .unwrap_or_else(|_| "15".to_string())
                .parse()
                .expect("ACCESS_TOKEN_EXPIRY_MINUTES must be a number"),
            refresh_token_expiry_days: std::env::var("REFRESH_TOKEN_EXPIRY_DAYS")
                .unwrap_or_else(|_| "7".to_string())
                .parse()
                .expect("REFRESH_TOKEN_EXPIRY_DAYS must be a number"),
            cors_origin: std::env::var("CORS_ORIGIN").unwrap_or_else(|_| "*".to_string()),
            rate_limit_requests: std::env::var("RATE_LIMIT_REQUESTS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .expect("RATE_LIMIT_REQUESTS must be a number"),
            rate_limit_seconds: std::env::var("RATE_LIMIT_SECONDS")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .expect("RATE_LIMIT_SECONDS must be a number"),
            request_timeout_seconds: std::env::var("REQUEST_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "20".to_string())
                .parse()
                .expect("REQUEST_TIMEOUT_SECONDS must be a number"),
            db_pool_max: std::env::var("DB_POOL_MAX")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .expect("DB_POOL_MAX must be a number"),
        }
    }
}

// ============================================================================
// APPLICATION STATE
// ============================================================================

/// Shared application state passed to all handlers
/// Contains database connection pool and configuration
#[derive(Clone)]
struct AppState {
    db_pool: PgPool,
    config: Config,
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
    access_token_expiry_minutes: i64,
) -> Result<(String, Claims), (StatusCode, String)> {
    let now = Utc::now();
    let access_exp = now + ChronoDuration::minutes(access_token_expiry_minutes);
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
/// Hash a password using Argon2 (runs on blocking thread)
async fn hash_password(password: String) -> Result<String, (StatusCode, String)> {
    tokio::task::spawn_blocking(move || {
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
    })
    .await
    .map_err(|e| {
        error!(%e, "Task join error");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error".to_string(),
        )
    })?
}

/// Verify a password against a stored hash
/// Verify a password against a stored hash (runs on blocking thread)
async fn verify_password(
    password: String,
    password_hash: String,
) -> Result<bool, (StatusCode, String)> {
    tokio::task::spawn_blocking(move || {
        let parsed_hash = match PasswordHash::new(&password_hash) {
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
    })
    .await
    .map_err(|e| {
        error!(%e, "Task join error");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error".to_string(),
        )
    })?
}

/// Generate and store a new refresh token
async fn create_refresh_token(
    user_id: i32,
    db_pool: &PgPool,
    refresh_token_expiry_days: i64,
) -> Result<String, (StatusCode, String)> {
    let now = Utc::now();
    let refresh_token = uuid::Uuid::new_v4().to_string();
    let refresh_exp = now + ChronoDuration::days(refresh_token_expiry_days);

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
            if match verify_password(payload.password.clone(), user.password.clone()).await {
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
                let (access_token, _claims) = match generate_access_token(
                    &user.username,
                    user.id,
                    &state.config.jwt_secret,
                    state.config.access_token_expiry_minutes,
                ) {
                    Ok(result) => result,
                    Err((status, msg)) => {
                        return (status, msg).into_response();
                    }
                };

                // Generate refresh token using helper
                let refresh_token = match create_refresh_token(
                    user.id,
                    &state.db_pool,
                    state.config.refresh_token_expiry_days,
                )
                .await
                {
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
    let password_hash = match hash_password(payload.password.clone()).await {
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
    let (access_token, _claims) = match generate_access_token(
        &username,
        user_id,
        &state.config.jwt_secret,
        state.config.access_token_expiry_minutes,
    ) {
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
            &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
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

/// Build CORS layer based on configuration
/// - "*" = permissive (allow all origins)
/// - specific URL = allow only that origin
fn build_cors_layer(cors_origin: &str) -> CorsLayer {
    use axum::http::Method;

    if cors_origin == "*" {
        CorsLayer::permissive()
    } else {
        CorsLayer::new()
            .allow_origin(
                cors_origin
                    .parse::<axum::http::HeaderValue>()
                    .expect("Invalid CORS_ORIGIN value"),
            )
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers(Any)
            .allow_credentials(true)
    }
}

/// Handle middleware errors (rate limit, timeout, etc.)
async fn handle_middleware_error(err: BoxError) -> (StatusCode, String) {
    if err.is::<tower::timeout::error::Elapsed>() {
        (StatusCode::REQUEST_TIMEOUT, "Request timed out".to_string())
    } else {
        (
            StatusCode::TOO_MANY_REQUESTS,
            format!("Request failed: {}", err),
        )
    }
}

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

    // Load configuration
    let config = Config::from_env();

    // Connect to PostgreSQL database
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    info!("⏳ Connecting to Database...");

    let db_pool = PgPoolOptions::new()
        .max_connections(config.db_pool_max)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_url)
        .await
        .expect("❌ Failed to connect to Postgres");

    info!("✅ Database connected successfully!");

    // Load configuration and create shared application state
    let config = Config::from_env();
    let app_state = AppState {
        db_pool,
        config: config.clone(),
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
        .layer(
            ServiceBuilder::new()
                // 1. Trace: Log every request/response (even if blocked by rate limit)
                .layer(TraceLayer::new_for_http())
                // 2. Compression: Compress response body (Gzip/Brotli)
                .layer(CompressionLayer::new())
                // 3. CORS: Allow cross-origin requests from frontend
                .layer(build_cors_layer(&config.cors_origin))
                // 4. HandleError: Catch errors from inner layers (RateLimit/Timeout) and convert to JSON
                .layer(HandleErrorLayer::new(handle_middleware_error))
                // 5. Buffer: Required for RateLimit (handles queuing/cloning of services)
                .layer(BufferLayer::new(1024))
                // 6. RateLimit: Limit requests per user/IP
                .layer(RateLimitLayer::new(
                    config.rate_limit_requests,
                    Duration::from_secs(config.rate_limit_seconds),
                ))
                // 7. Timeout: Abort request if processing takes too long
                .layer(TimeoutLayer::new(Duration::from_secs(
                    config.request_timeout_seconds,
                ))),
        );

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
