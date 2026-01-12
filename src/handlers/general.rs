use axum::{Json, response::IntoResponse};
use tracing::info;

/// Health check / welcome endpoint
pub async fn index() -> impl IntoResponse {
    info!("Index endpoint hit");
    Json(serde_json::json!({
        "status": "success",
        "message": "Welcome to the Index API",
    }))
}
