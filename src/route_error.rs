use anyhow::Error;
use axum::{
    body::{Bytes, Full, HttpBody},
    http::Response,
    response::IntoResponse,
};
use hyper::StatusCode;

pub struct RouteError(Error);

impl RouteError {
    pub fn into(self) -> Error {
        self.0
    }
}

impl From<Error> for RouteError {
    fn from(err: Error) -> Self {
        Self(err)
    }
}

impl IntoResponse for RouteError {
    type Body = Full<Bytes>;
    type BodyError = <Self::Body as HttpBody>::Error;

    fn into_response(self) -> Response<Self::Body> {
        tracing::error!("{}", self.0);
        (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response()
    }
}
