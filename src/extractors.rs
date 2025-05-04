use axum::{extract::{FromRequest, FromRequestParts}, response::IntoResponse};
use serde::Serialize;

use crate::error::Error;

// MAIN JSON EXTRACTOR
// ----------------------------------------------------------------------------
#[derive(FromRequest, Serialize)]
#[from_request(via(axum::Json), rejection(Error))]
pub struct Json<T>(pub T);

impl<T> IntoResponse for Json<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        axum::Json(self.0).into_response()
    }
}

// QUERY EXTRACTOR
// --------------------------------------------------------------------------------
#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Query), rejection(Error))]
pub struct Query<T>(pub T);

// PATH EXTRACTOR
// ---------------------------------------------------------------------------------
#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(Error))]
pub struct Path<T>(pub T);

// HOST
// #[derive(FromRequestParts)]
// #[from_request(via(axum_extra::extract::Host), rejection(Error))]
// pub struct Host(String);
