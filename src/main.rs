mod config;
mod handlers;
mod models;
mod routes;
mod state;
mod utils;

use axum::{BoxError, error_handling::HandleErrorLayer, http::StatusCode};
use dotenvy::dotenv;
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
use tracing::info;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use config::Config;
use state::AppState;

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

    // Connect to Redis/Dragonfly cache
    info!("⏳ Connecting to Cache (Dragonfly)...");
    let redis_client =
        redis::Client::open(config.redis_url.as_str()).expect("❌ Invalid REDIS_URL");
    let redis_conn = redis::aio::ConnectionManager::new(redis_client)
        .await
        .expect("❌ Failed to connect to Redis/Dragonfly");
    info!("✅ Cache connected successfully!");

    // Create cache wrapper
    let cache = utils::cache::Cache::new(redis_conn, config.cache_ttl_seconds);

    // Create shared application state
    let app_state = AppState {
        db_pool,
        cache,
        config: config.clone(),
    };

    // Configure routes and middleware
    let app = routes::create_router().with_state(app_state).layer(
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
