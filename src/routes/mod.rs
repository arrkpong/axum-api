use crate::handlers::general;
use crate::state::AppState;
use axum::{Router, routing::get};

pub mod auth;
pub mod benchmark;
pub mod user;

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/", get(general::index))
        .merge(auth::routes())
        .merge(user::routes())
    // .merge(benchmark::routes())
}
