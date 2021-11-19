use std::{future::Future, ops::Deref, pin::Pin};

use anyhow::{anyhow, Context};
use axum::extract::{FromRequest, RequestParts};
use deadpool_postgres::{Object, Pool};

use crate::route_error::RouteError;

pub struct DbConn(Object);

impl Deref for DbConn {
    type Target = Object;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<B: Send> FromRequest<B> for DbConn {
    type Rejection = RouteError;

    fn from_request<'a, 'f>(
        req: &'a mut RequestParts<B>,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Rejection>> + Send + 'f>>
    where
        'a: 'f,
    {
        Box::pin(async move {
            let pool = req
                .extensions()
                .ok_or_else(|| anyhow!("cannot obtain extensions"))?
                .get::<Pool>()
                .ok_or_else(|| anyhow!("cannot obtain database pool"))?;
            let conn = pool
                .get()
                .await
                .context("cannot obtain database connection")?;
            Ok(DbConn(conn))
        })
    }
}
