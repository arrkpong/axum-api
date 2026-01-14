#![allow(dead_code)]

use axum::{Json, response::IntoResponse};
use serde_json::json;
use tokio::time::{Duration, sleep};

// CPU-bound: Calculate Fibonacci (naive recursion to simulate heavy load)
// Using n=30 should bridge the gap to make it measurable but not hang forever.
fn fib(n: u64) -> u64 {
    if n <= 1 {
        return n;
    }
    fib(n - 1) + fib(n - 2)
}

pub async fn cpu_bound() -> impl IntoResponse {
    let start = std::time::Instant::now();
    let n = 30;
    let result = fib(n);
    let duration = start.elapsed();

    Json(json!({
        "test": "cpu_bound",
        "description": format!("Fibonacci({})", n),
        "result": result,
        "duration_ms": duration.as_millis()
    }))
}

// I/O-bound: Simulate DB query or external API call (Async Sleep)
pub async fn io_bound() -> impl IntoResponse {
    let start = std::time::Instant::now();
    sleep(Duration::from_millis(50)).await;
    let duration = start.elapsed();

    Json(json!({
        "test": "io_bound",
        "description": "Sleep 50ms",
        "duration_ms": duration.as_millis()
    }))
}

// Database-bound: Real DB Query
#[derive(sqlx::FromRow, serde::Serialize)]
struct BenchmarkUser {
    id: i32,
    username: String,
    email: Option<String>,
}

pub async fn db_read(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();

    // Fetch 50 users
    let users =
        sqlx::query_as::<_, BenchmarkUser>("SELECT id, username, email FROM auth_users LIMIT 50")
            .fetch_all(&state.db_pool)
            .await
            .unwrap_or_else(|_| vec![]);

    let duration = start.elapsed();

    Json(json!({
        "test": "db_read",
        "description": "SELECT * FROM auth_users LIMIT 50",
        "result_count": users.len(),
        "duration_ms": duration.as_millis()
    }))
}

pub async fn db_write(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();
    let unique_id = uuid::Uuid::new_v4().to_string();
    let email = format!("bm_{}@test.com", unique_id);
    let username = format!("bm_{}", unique_id);
    let password = "benchmark_password_123";

    // Simulate Django's workload: Hash password with Argon2 (CPU intensive)
    // We use the same parameters as default Django (Argon2id) where possible,
    // or reasonable defaults for production security.
    let salt =
        argon2::password_hash::SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    let argon2 = argon2::Argon2::default();
    let password_hash = argon2::PasswordHasher::hash_password(&argon2, password.as_bytes(), &salt)
        .map(|pool| pool.to_string())
        .unwrap_or_else(|_| "hash_error".to_string());

    // Insert dummy user
    // Assuming created_at/updated_at have defaults or are handled by DB triggers
    let _ = sqlx::query(
        "INSERT INTO auth_users (username, email, password_hash, role, is_active) VALUES ($1, $2, $3, 'user', true)"
    )
    .bind(username)
    .bind(email)
    .bind(password_hash)
    .execute(&state.db_pool)
    .await;

    let duration = start.elapsed();

    Json(json!(
        {
            "test": "db_write",
            "description": "INSERT 1 row into auth_users (with Argon2 Hashing)",
            "duration_ms": duration.as_millis()
        }
    ))
}
