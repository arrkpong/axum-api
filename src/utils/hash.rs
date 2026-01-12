use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::http::StatusCode;
use tracing::error;

/// Hash a password using Argon2 (runs on blocking thread)
pub async fn hash_password(password: String) -> Result<String, (StatusCode, String)> {
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        match argon2.hash_password(password.as_bytes(), &salt) {
            Ok(hash) => Ok(hash.to_string()),
            Err(e) => {
                error!(%e, "Password hashing failed");
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to hash password".to_string(),
                ))
            }
        }
    })
    .await
    .map_err(|e| {
        error!(%e, "Task join error");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error".to_string(),
        )
    })?
}

/// Verify a password against a stored hash (runs on blocking thread)
pub async fn verify_password(
    password: String,
    password_hash: String,
) -> Result<bool, (StatusCode, String)> {
    tokio::task::spawn_blocking(move || {
        let parsed_hash = match PasswordHash::new(&password_hash) {
            Ok(hash) => hash,
            Err(e) => {
                error!(%e, "Failed to parse stored password hash");
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to verify password".to_string(),
                ));
            }
        };

        let argon2 = Argon2::default();
        Ok(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    })
    .await
    .map_err(|e| {
        error!(%e, "Task join error");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error".to_string(),
        )
    })?
}
