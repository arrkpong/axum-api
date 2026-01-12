use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use sqlx::Row;
use tracing::{error, info, instrument, warn};

use crate::{
    models::{
        auth::{LoginPayload, RefreshPayload, RegisterPayload},
        user::AuthUser,
    },
    state::AppState,
    utils::{
        hash::{hash_password, verify_password},
        jwt::{create_refresh_token, generate_access_token},
        validation::ValidatedJson,
    },
};

/// User login handler
#[instrument(skip(state, payload), fields(username = %payload.username))]
pub async fn login(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<LoginPayload>,
) -> impl IntoResponse {
    info!("Login endpoint hit");

    // Query user from database
    let select_user = sqlx::query!(
        "SELECT id, username, password FROM auth_users WHERE username = $1",
        &payload.username,
    )
    .fetch_optional(&state.db_pool)
    .await;

    match select_user {
        Ok(Some(user)) => {
            // Verify password using helper
            if match verify_password(payload.password.clone(), user.password.clone()).await {
                Ok(valid) => valid,
                Err((status, msg)) => {
                    return (
                        status,
                        Json(serde_json::json!({"status": "error", "message": msg})),
                    )
                        .into_response();
                }
            } {
                // Generate access token using helper
                let (access_token, _claims) = match generate_access_token(
                    &user.username,
                    user.id,
                    &state.config.jwt_secret,
                    state.config.access_token_expiry_minutes,
                ) {
                    Ok(result) => result,
                    Err((status, msg)) => {
                        return (status, msg).into_response();
                    }
                };

                // Generate refresh token using helper
                let refresh_token = match create_refresh_token(
                    user.id,
                    &state.db_pool,
                    state.config.refresh_token_expiry_days,
                )
                .await
                {
                    Ok(token) => token,
                    Err((status, msg)) => {
                        return (
                            status,
                            Json(serde_json::json!({"status": "error", "message": msg})),
                        )
                            .into_response();
                    }
                };

                info!(user_id = user.id, "Login successful");
                (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "status": "success",
                        "message": "Login successful",
                        "access_token": access_token,
                        "refresh_token": refresh_token,
                        "expires_in": 900,
                        "user_id": user.id
                    })),
                )
                    .into_response()
            } else {
                warn!("Login failed: Incorrect password");
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({"status": "error", "message": "Invalid credentials"})),
                )
                    .into_response()
            }
        }

        Ok(None) => {
            warn!("Login failed: User not found");
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"status": "error", "message": "Invalid credentials"})),
            )
                .into_response()
        }

        Err(e) => {
            error!(%e, "Database query failed during login");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": "Failed to fetch user"})),
            )
                .into_response()
        }
    }
}

/// User registration handler
#[instrument(skip(state, payload), fields(username = %payload.username))]
pub async fn register(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<RegisterPayload>,
) -> impl IntoResponse {
    info!("Register endpoint hit");

    // Generate random salt and hash password using helper
    let password_hash = match hash_password(payload.password.clone()).await {
        Ok(hash) => hash,
        Err((status, msg)) => {
            return (
                status,
                Json(serde_json::json!({"status": "error", "message": msg})),
            )
                .into_response();
        }
    };

    // Insert new user into database
    let insert_user = sqlx::query(
        "INSERT INTO auth_users (username, email, phone, password) VALUES ($1, $2, $3, $4)",
    )
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&payload.phone)
    .bind(&password_hash)
    .execute(&state.db_pool)
    .await;

    match insert_user {
        Ok(_) => {
            info!("User registered successfully");
            (
                StatusCode::CREATED,
                Json(
                    serde_json::json!({"status": "success", "message": "User registered successfully"}),
                ),
            ).into_response()
        }
        Err(e) => {
            if let Some(db_error) = e.as_database_error()
                && db_error.is_unique_violation()
            {
                warn!(%db_error, "Registration failed: Unique constraint violation");

                return match db_error.constraint() {
                    Some("auth_users_username_key") => (
                        StatusCode::CONFLICT,
                        Json(serde_json::json!({"status": "error", "message": "Username already exists"})),
                    ).into_response(),
                    _ => (
                        StatusCode::CONFLICT,
                        Json(serde_json::json!({"status": "error", "message": "Unique constraint violation"})),
                    ).into_response(),
                };
            }

            error!(%e, "Database insertion failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": "Failed to register user"})),
            )
                .into_response()
        }
    }
}

/// Logout handler - adds token to blacklist and removes refresh tokens
pub async fn logout(State(state): State<AppState>, auth_user: AuthUser) -> impl IntoResponse {
    info!("Logout endpoint hit for user: {}", auth_user.username);

    // Insert access token JTI into blacklist
    let blacklist_result = sqlx::query(
        "INSERT INTO auth_token_blacklist (token_jti, user_id, expires_at) VALUES ($1, $2, $3) ON CONFLICT (token_jti) DO NOTHING"
    )
    .bind(&auth_user.jti)
    .bind(auth_user.user_id)
    .bind(auth_user.exp)
    .execute(&state.db_pool)
    .await;

    // Delete all refresh tokens for this user
    let delete_result = sqlx::query("DELETE FROM auth_refresh_tokens WHERE user_id = $1")
        .bind(auth_user.user_id)
        .execute(&state.db_pool)
        .await;

    if blacklist_result.is_err() || delete_result.is_err() {
        error!("Failed to complete logout");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"status": "error", "message": "Logout failed"})),
        )
            .into_response();
    }

    info!("Logout successful - tokens revoked");
    Json(serde_json::json!({
        "status": "success",
        "message": "Logout successful",
    }))
    .into_response()
}

/// Refresh token handler - exchange refresh token for new access token
pub async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<RefreshPayload>,
) -> impl IntoResponse {
    info!("Refresh token endpoint hit");

    // Find refresh token in DB
    let token_record =
        sqlx::query("SELECT user_id, expires_at FROM auth_refresh_tokens WHERE token = $1")
            .bind(&payload.refresh_token)
            .fetch_optional(&state.db_pool)
            .await;

    let row = match token_record {
        Ok(Some(row)) => row,
        Ok(None) => {
            warn!("Refresh token not found");
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"status": "error", "message": "Invalid refresh token"})),
            )
                .into_response();
        }
        Err(e) => {
            error!(%e, "Database error during refresh");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": "Refresh failed"})),
            )
                .into_response();
        }
    };

    let user_id: i32 = row.get("user_id");
    let expires_at: chrono::DateTime<chrono::Utc> = row.get("expires_at");

    // Check if refresh token is expired
    if expires_at < chrono::Utc::now() {
        // Delete expired token
        let _ = sqlx::query("DELETE FROM auth_refresh_tokens WHERE token = $1")
            .bind(&payload.refresh_token)
            .execute(&state.db_pool)
            .await;
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"status": "error", "message": "Refresh token expired"})),
        )
            .into_response();
    }

    // Get username for claims
    let user_record = sqlx::query("SELECT username FROM auth_users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.db_pool)
        .await;

    let username: String = match user_record {
        Ok(Some(row)) => row.get("username"),
        _ => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": "User not found"})),
            )
                .into_response();
        }
    };

    // Generate new access token using helper
    let (access_token, _claims) = match generate_access_token(
        &username,
        user_id,
        &state.config.jwt_secret,
        state.config.access_token_expiry_minutes,
    ) {
        Ok(result) => result,
        Err((status, msg)) => {
            return (status, msg).into_response();
        }
    };

    info!(user_id = user_id, "Token refreshed successfully");
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "success",
            "access_token": access_token,
            "expires_in": 900
        })),
    )
        .into_response()
}
