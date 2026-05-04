use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use serde::Serialize;

use sqlx::Error as SqlxError;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    DbError(String),
    TooManyRequests(String)
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl From<SqlxError> for AppError {
    fn from(err: SqlxError) -> Self {
        AppError::DbError(err.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(message) => (StatusCode::NOT_FOUND, message),
            AppError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            AppError::DbError(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
            AppError::TooManyRequests(message)=> (StatusCode::TOO_MANY_REQUESTS, message)
        };

        let body = Json(ErrorResponse { error: message });

        (status, body).into_response()
    }
}
