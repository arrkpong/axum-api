#![allow(dead_code)]

use crate::handlers::benchmark;
use crate::state::AppState;
use axum::{Router, routing::get};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/benchmark/cpu", get(benchmark::cpu_bound))
        .route("/benchmark/io", get(benchmark::io_bound))
        .route("/benchmark/db-read", get(benchmark::db_read))
        .route("/benchmark/db-write", get(benchmark::db_write)) // GET for easier benchmarking with ab
}
