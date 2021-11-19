use std::{future::Future, pin::Pin};

use axum::{
    body::{Bytes, Full, HttpBody},
    extract::{FromRequest, RequestParts},
    http,
    response::IntoResponse,
};
use headers::{authorization::Bearer, Authorization, HeaderMapExt};
use hyper::StatusCode;
use uuid::Uuid;

use crate::db_conn::DbConn;

pub struct Authentication;

pub enum AuthenticationRejection {
    Db,
    Unauthorized,
}

impl IntoResponse for AuthenticationRejection {
    type Body = Full<Bytes>;
    type BodyError = <Self::Body as HttpBody>::Error;

    fn into_response(self) -> http::Response<Self::Body> {
        match self {
            AuthenticationRejection::Db => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response()
            }
            AuthenticationRejection::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "unauthorized").into_response()
            }
        }
    }
}

impl<B: Send> FromRequest<B> for Authentication {
    type Rejection = AuthenticationRejection;

    fn from_request<'a, 'f>(
        req: &'a mut RequestParts<B>,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Rejection>> + Send + 'f>>
    where
        'a: 'f,
    {
        Box::pin(async move {
            let bearer = req
                .headers()
                .ok_or(AuthenticationRejection::Unauthorized)?
                .typed_get::<Authorization<Bearer>>()
                .ok_or(AuthenticationRejection::Unauthorized)?;
            let token = bearer.0.token();
            let uuid = Uuid::parse_str(token).map_err(|e| {
                tracing::error!("{}", e);
                AuthenticationRejection::Unauthorized
            })?;
            let conn = DbConn::from_request(req).await.map_err(|e| {
                let e = e.into();
                tracing::error!("{}", e);
                AuthenticationRejection::Db
            })?;
            let r = conn
                .query_one(
                    "select exists (select * from access_tokens where id = $1)",
                    &[&uuid],
                )
                .await
                .map_err(|e| {
                    tracing::error!("{}", e);
                    AuthenticationRejection::Db
                })?;
            if r.get::<_, bool>(0) {
                Ok(Authentication)
            } else {
                Err(AuthenticationRejection::Unauthorized)
            }
        })
    }
}
