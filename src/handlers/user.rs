use axum::{Json, response::IntoResponse};
use tracing::info;

use crate::models::user::AuthUser;

/// Protected profile endpoint
pub async fn profile(auth_user: AuthUser) -> impl IntoResponse {
    info!("Profile endpoint accessed by: {}", auth_user.username);
    Json(serde_json::json!({
        "status": "success",
        "message": format!("Welcome back, {}!", auth_user.username),
        "user_id": auth_user.user_id,
        "username": auth_user.username,
        "email": "TODO", // We will need to fetch this from DB if needed
        "phone": "TODO"
    }))
}
