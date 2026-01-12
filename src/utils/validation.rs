use axum::{
    Json,
    extract::{FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::de::DeserializeOwned;
use validator::Validate;

/// Custom extractor that validates the extracted JSON
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // 1. Extract JSON using axum's built-in extractor
        let Json(value) = Json::<T>::from_request(req, state).await.map_err(|err| {
            let message = err.body_text();
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "status": "error",
                    "message": "Invalid JSON format",
                    "details": message
                })),
            )
                .into_response()
        })?;

        // 2. Validate the struct
        value.validate().map_err(|err| {
            let errors = err
                .field_errors()
                .iter()
                .map(|(field, errors)| {
                    let messages: Vec<String> = errors
                        .iter()
                        .map(|e| {
                            e.message
                                .as_ref()
                                .unwrap_or(&std::borrow::Cow::Borrowed("Invalid value"))
                                .to_string()
                        })
                        .collect();
                    (field.to_string(), messages)
                })
                .collect::<std::collections::HashMap<String, Vec<String>>>();

            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "status": "error",
                    "message": "Validation failed",
                    "errors": errors
                })),
            )
                .into_response()
        })?;

        // 3. Return validated struct
        Ok(ValidatedJson(value))
    }
}
