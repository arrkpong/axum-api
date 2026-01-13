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
