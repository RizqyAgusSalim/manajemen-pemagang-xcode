use axum::{
    response::{Response, IntoResponse},
    http::StatusCode,
    Json,
};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

#[derive(Debug)]
pub enum AppError {
    Database(sqlx::Error),
    Unauthorized,
    TokenInvalid,
    Internal,
    BadRequest(String),
    NotFound(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // ✅ LOG ERROR DETAIL KE TERMINAL
        match &self {
            AppError::Database(e) => tracing::error!("💥 DATABASE ERROR: {:?}", e),
            AppError::Unauthorized => tracing::warn!("⚠️ UNAUTHORIZED: Access denied"),
            AppError::TokenInvalid => tracing::warn!("⚠️ TOKEN INVALID: JWT verification failed"),
            AppError::Internal => tracing::error!("❌ INTERNAL ERROR: Generic internal error - check handler logs for specific context"),
            AppError::BadRequest(msg) => tracing::warn!("⚠️ BAD REQUEST: {}", msg),
            AppError::NotFound(msg) => tracing::warn!("⚠️ NOT FOUND: {}", msg),
        }

        let (status, error_msg, details) = match self {
            AppError::Database(e) => {
                let detail = format!("{:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string(), Some(detail))
            },
            AppError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "Unauthorized".to_string(), None)
            },
            AppError::TokenInvalid => {
                (StatusCode::UNAUTHORIZED, "Token invalid or expired".to_string(), None)
            },
            AppError::Internal => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Terjadi kesalahan internal".to_string(), None)
            },
            AppError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, msg, None)
            },
            AppError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, msg, None)
            },
        };

        (status, Json(ErrorResponse { 
            error: error_msg, 
            details 
        })).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err)
    }
}