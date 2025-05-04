use std::fmt::Display;

use axum::{extract::rejection::{JsonRejection, PathRejection, QueryRejection}, http::StatusCode, response::{IntoResponse, Response}};
use serde::Serialize;
use tokio::time::error::Elapsed;
use url::ParseError;
use utoipa::ToSchema;

use crate::extractors::Json;

pub type Result<T> = core::result::Result<T, Error>;

/// Represents all errors the API can return
#[derive(thiserror::Error, Debug, ToSchema)]
pub enum Error {
    // Short link redirection
    #[error("A link with the provided ID '{0}' could not be found")]
    LinkNotFound(String),

    // Short link generation
    #[error("The provided custom link ID is already in use: {0}")]
    LinkIdNotUnique(String),
    #[error("The provided custom link ID is not valid: {0}")]
    LinkIdNotValid(String),
    #[error("Malformed URL: {0}")]
    MalformedURL(String),
    #[error("Only URLs with valid hosts are accepted: {0}")]
    URLWithoutHost(String),
    #[error("URLs with the same host as this service are forbidden: {0}")]
    URLWithMatchingHosts(String),

    // Other errors
    #[error("Route not found")]
    RouteNotFound,
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    // Avoid exposing details about internal server errors to the client
    #[error("Something went wrong")]
    Internal(String),
}

/// For serialising error response into a specific format
#[derive(Serialize, Debug, ToSchema)]
pub struct ErrorResponse {
    pub message: String,
}
impl ErrorResponse {
    pub fn new(message: impl Display) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match &self {
            // Redirection
            Self::LinkNotFound(_) => StatusCode::NOT_FOUND,

            // Creation
            Self::LinkIdNotUnique(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::MalformedURL(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::URLWithoutHost(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::URLWithMatchingHosts(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::LinkIdNotValid(_) => StatusCode::UNPROCESSABLE_ENTITY,

            Self::RouteNotFound => StatusCode::NOT_FOUND,
            Self::InvalidRequest(_) => StatusCode::BAD_REQUEST,

            Self::Internal(s) => {
                tracing::error!("Internal server error: {s}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        (status, Json(ErrorResponse::from(self))).into_response()
    }
}

impl From<Error> for ErrorResponse {
    fn from(value: Error) -> Self {
        Self::new(value.to_string())
    }
}

impl From<JsonRejection> for Error {
    fn from(rejection: JsonRejection) -> Self {
        Self::InvalidRequest(rejection.to_string())
    }
}

impl From<QueryRejection> for Error {
    fn from(rejection: QueryRejection) -> Self {
        Self::InvalidRequest(rejection.to_string())
    }
}

impl From<PathRejection> for Error {
    fn from(rejection: PathRejection) -> Self {
        Self::InvalidRequest(rejection.to_string())
    }
}

impl From<Elapsed> for Error {
    fn from(elapsed: Elapsed) -> Self {
        Self::Internal(elapsed.to_string())
    }
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        Self::Internal(e.to_string())
    }
}

impl From<url::ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Self::MalformedURL(e.to_string())
    }
}
