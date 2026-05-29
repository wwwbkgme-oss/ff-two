use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use errors::AppError;

/// Newtype-Wrapper für AppError — nötig wegen Rust Orphan-Regeln.
/// `IntoResponse` darf nur im selben Crate wie der Trait oder der Typ impl sein.
pub struct ApiError(pub AppError);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code) = match &self.0 {
            AppError::NotFound(_)     => (StatusCode::NOT_FOUND,            "NOT_FOUND"),
            AppError::Conflict(_)     => (StatusCode::CONFLICT,             "CONFLICT"),
            AppError::BadRequest(_)   => (StatusCode::BAD_REQUEST,          "BAD_REQUEST"),
            AppError::Unauthorized    => (StatusCode::UNAUTHORIZED,          "UNAUTHORIZED"),
            AppError::Forbidden       => (StatusCode::FORBIDDEN,             "FORBIDDEN"),
            AppError::Sandbox(_)      => (StatusCode::INTERNAL_SERVER_ERROR, "SANDBOX_ERROR"),
            AppError::Agent(_)        => (StatusCode::INTERNAL_SERVER_ERROR, "AGENT_ERROR"),
            AppError::SecurityScan(_) => (StatusCode::UNPROCESSABLE_ENTITY,  "SECURITY_SCAN_FAIL"),
            AppError::Deployment(_)   => (StatusCode::INTERNAL_SERVER_ERROR, "DEPLOYMENT_ERROR"),
            AppError::Internal(_)     => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
        };
        tracing::error!(error = %self.0, status = %status, code, "request error");
        (status, Json(json!({ "error": { "code": code, "message": self.0.to_string() } }))).into_response()
    }
}

impl From<AppError> for ApiError {
    fn from(e: AppError) -> Self { Self(e) }
}

/// Handler-Ergebnis-Alias.
pub type ApiResult<T> = std::result::Result<T, ApiError>;
