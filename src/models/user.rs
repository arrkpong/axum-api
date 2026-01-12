use crate::state::AppState;
use axum::{
    Json,
    extract::FromRequestParts,
    http::{StatusCode, header::AUTHORIZATION, request::Parts},
};
use chrono::{DateTime, Utc};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};

/// JWT token claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub user_id: i32,
    pub jti: String,
    pub exp: usize,
    pub iat: usize,
}

/// Authenticated user extracted from JWT token
pub struct AuthUser {
    pub username: String,
    pub user_id: i32,
    pub jti: String,
    pub exp: DateTime<Utc>,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Step 1: Extract Authorization header
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"status": "error", "message": "Missing Authorization header"})),
            ))?;

        // Step 2: Validate "Bearer " prefix
        if !auth_header.starts_with("Bearer ") {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"status": "error", "message": "Invalid header format"})),
            ));
        }

        // Step 3: Extract and verify JWT token
        let token = &auth_header[7..];
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"status": "error", "message": "Invalid token"})),
            )
        })?;

        // Step 4: Check if token is blacklisted
        // We need to access the database to check the blacklist
        let blacklist_check = sqlx::query(
            "SELECT 1 FROM auth_token_blacklist WHERE token_jti = $1",
        )
        .bind(&token_data.claims.jti)
        .fetch_optional(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": "Internal server error"})),
            )
        })?;

        if blacklist_check.is_some() {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"status": "error", "message": "Token has been revoked"})),
            ));
        }

        // Convert exp timestamp to DateTime
        let exp_dt =
            DateTime::from_timestamp(token_data.claims.exp as i64, 0).unwrap_or_else(Utc::now);

        Ok(AuthUser {
            username: token_data.claims.sub,
            user_id: token_data.claims.user_id,
            jti: token_data.claims.jti,
            exp: exp_dt,
        })
    }
}
