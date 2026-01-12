use crate::{handlers::user, state::AppState};
use axum::{Router, routing::get};

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/v1/profile", get(user::profile))
}
