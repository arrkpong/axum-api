use axum::http::StatusCode;
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use sqlx::PgPool;
use tracing::error;

use crate::models::user::Claims;

/// Generate a new access token for the given user
/// Returns (access_token, claims) or error response
pub fn generate_access_token(
    username: &str,
    user_id: i32,
    jwt_secret: &str,
    access_token_expiry_minutes: i64,
) -> Result<(String, Claims), (StatusCode, String)> {
    let now = Utc::now();
    let access_exp = now + Duration::minutes(access_token_expiry_minutes);
    let claims = Claims {
        sub: username.to_string(),
        user_id,
        jti: uuid::Uuid::new_v4().to_string(),
        iat: now.timestamp() as usize,
        exp: access_exp.timestamp() as usize,
    };

    match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    ) {
        Ok(token) => Ok((token, claims)),
        Err(e) => {
            error!(%e, "Failed to create access token");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Token creation failed".to_string(),
            ))
        }
    }
}

/// Generate and store a new refresh token
pub async fn create_refresh_token(
    user_id: i32,
    db_pool: &PgPool,
    refresh_token_expiry_days: i64,
) -> Result<String, (StatusCode, String)> {
    let now = Utc::now();
    let refresh_token = uuid::Uuid::new_v4().to_string();
    let refresh_exp = now + Duration::days(refresh_token_expiry_days);

    // Store refresh token in DB
    let store_result = sqlx::query(
        "INSERT INTO auth_refresh_tokens (user_id, token, expires_at) VALUES ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(&refresh_token)
    .bind(refresh_exp)
    .execute(db_pool)
    .await;

    match store_result {
        Ok(_) => Ok(refresh_token),
        Err(e) => {
            error!(%e, "Failed to store refresh token");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Login failed".to_string(),
            ))
        }
    }
}
