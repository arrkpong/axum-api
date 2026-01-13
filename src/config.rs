use std::env;

/// Application configuration loaded from environment variables
#[derive(Clone)]
pub struct Config {
    /// JWT secret key for signing tokens
    pub jwt_secret: String,
    /// Access token expiry in minutes
    pub access_token_expiry_minutes: i64,
    /// Refresh token expiry in days
    pub refresh_token_expiry_days: i64,
    /// CORS allowed origin ("*" for permissive)
    pub cors_origin: String,
    /// Rate limit: requests per window
    pub rate_limit_requests: u64,
    /// Rate limit: window in seconds
    pub rate_limit_seconds: u64,
    /// Request timeout in seconds
    pub request_timeout_seconds: u64,
    /// Database pool max connections
    pub db_pool_max: u32,
    /// Redis/Dragonfly connection URL
    pub redis_url: String,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            access_token_expiry_minutes: env::var("ACCESS_TOKEN_EXPIRY_MINUTES")
                .unwrap_or_else(|_| "15".to_string())
                .parse()
                .expect("ACCESS_TOKEN_EXPIRY_MINUTES must be a number"),
            refresh_token_expiry_days: env::var("REFRESH_TOKEN_EXPIRY_DAYS")
                .unwrap_or_else(|_| "7".to_string())
                .parse()
                .expect("REFRESH_TOKEN_EXPIRY_DAYS must be a number"),
            cors_origin: env::var("CORS_ORIGIN").unwrap_or_else(|_| "*".to_string()),
            rate_limit_requests: env::var("RATE_LIMIT_REQUESTS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .expect("RATE_LIMIT_REQUESTS must be a number"),
            rate_limit_seconds: env::var("RATE_LIMIT_SECONDS")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .expect("RATE_LIMIT_SECONDS must be a number"),
            request_timeout_seconds: env::var("REQUEST_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "20".to_string())
                .parse()
                .expect("REQUEST_TIMEOUT_SECONDS must be a number"),
            db_pool_max: env::var("DB_POOL_MAX")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .expect("DB_POOL_MAX must be a number"),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            cache_ttl_seconds: env::var("CACHE_TTL_SECONDS")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .expect("CACHE_TTL_SECONDS must be a number"),
        }
    }
}
