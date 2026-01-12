use serde::Deserialize;
use validator::Validate;

/// Payload for user login requests
#[derive(Deserialize, Validate)]
pub struct LoginPayload {
    #[validate(length(min = 3, message = "Username must be at least 3 characters"))]
    pub username: String,
    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    pub password: String,
}

/// Payload for user registration requests
#[derive(Deserialize, Validate)]
pub struct RegisterPayload {
    #[validate(length(min = 3, message = "Username must be at least 3 characters"))]
    pub username: String,
    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    pub password: String,
    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,
    #[validate(length(min = 10, message = "Phone must be at least 10 characters"))]
    pub phone: Option<String>,
}

/// Payload for token refresh requests
#[derive(Deserialize)]
pub struct RefreshPayload {
    pub refresh_token: String,
}
