use crate::handlers::general;
use crate::state::AppState;
use axum::{Router, routing::get};

pub mod auth;
pub mod benchmark;
pub mod user;

pub fn create_router(enable_benchmark_routes: bool) -> Router<AppState> {
    let router = Router::new()
        .route("/", get(general::index))
        .merge(auth::routes())
        .merge(user::routes());

    if enable_benchmark_routes {
        router.merge(benchmark::routes())
    } else {
        router
    }
}
