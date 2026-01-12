use crate::{handlers::auth, state::AppState};
use axum::{Router, routing::post};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/login", post(auth::login))
        .route("/api/v1/register", post(auth::register))
        .route("/api/v1/logout", post(auth::logout))
        .route("/api/v1/refresh", post(auth::refresh))
}
