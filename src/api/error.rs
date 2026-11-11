use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::error;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug)]
pub enum ApiError {
    NotFound,
    BadRequest,
    Internal(post_archiver::error::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::NotFound => StatusCode::NOT_FOUND.into_response(),
            ApiError::BadRequest => StatusCode::BAD_REQUEST.into_response(),
            ApiError::Internal(err) => {
                error!("{err}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

impl From<post_archiver::error::Error> for ApiError {
    fn from(err: post_archiver::error::Error) -> Self {
        ApiError::Internal(err)
    }
}

impl From<rusqlite::Error> for ApiError {
    fn from(err: rusqlite::Error) -> Self {
        ApiError::Internal(err.into())
    }
}
