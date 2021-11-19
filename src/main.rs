use std::{
    env,
    net::{IpAddr, SocketAddr, TcpListener},
};

use anyhow::{Context, Result};
use axum::{routing::post, AddExtensionLayer, Json, Router, Server};
use deadpool_postgres::{Config, ManagerConfig, PoolConfig, RecyclingMethod};
use tokio::runtime::Builder;
use tokio_postgres::NoTls;
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tracing::Level;

use self::{
    authentication::Authentication,
    db_conn::DbConn,
    models::{Stats, Stats2, Tabs},
    route_error::RouteError,
};

mod authentication;
mod db_conn;
mod models;
mod route_error;

async fn stats(
    _authorization: Authentication,
    conn: DbConn,
    stats: Json<Stats>,
) -> Result<(), RouteError> {
    conn.execute(
        "call insert_stats($1, $2, $3)",
        &[&stats.time, &stats.temperature, &stats.humidity],
    )
    .await
    .context("cannot save data")?;
    Ok(())
}

async fn stats2(
    _authorization: Authentication,
    conn: DbConn,
    stats: Json<Stats2>,
) -> Result<(), RouteError> {
    conn.execute(
        "call insert_stats2($1, $2, $3)",
        &[&stats.time, &stats.temperature, &(stats.co2 as i16)],
    )
    .await
    .context("cannot save data")?;
    Ok(())
}

async fn tabs(
    _authorization: Authentication,
    conn: DbConn,
    tabs: Json<Tabs>,
) -> Result<(), RouteError> {
    conn.execute(
        "insert into tabs(time, tabs) values($1, $2)",
        &[&tabs.time, &(tabs.tabs as i16)],
    )
    .await
    .context("cannot save data")?;
    Ok(())
}

async fn run() -> Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "stats_logger=info,tower_http=info")
    }
    tracing_subscriber::fmt::init();

    let address = "127.0.0.1";
    let port = 8080;
    let addr = SocketAddr::new(
        address
            .parse::<IpAddr>()
            .context("cannot parse listen address")?,
        port,
    );

    let mut cfg = Config::new();
    cfg.host = env::var("DB_HOST").ok();
    if let Some(port) = env::var("DB_PORT").ok() {
        cfg.port = Some(port.parse().context("invalid `DB_PORT`")?);
    }
    cfg.user = env::var("DB_USER").ok();
    cfg.password = env::var("DB_PASS").ok();
    cfg.dbname = env::var("DB_NAME").ok();
    cfg.application_name = Some("stats-logger".to_string());
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    cfg.pool = Some(PoolConfig::new(1));

    let pool = cfg
        .create_pool(Some(deadpool_postgres::Runtime::Tokio1), NoTls)
        .context("cannot create connection pool")?;

    let app = Router::new()
        .route("/stats", post(stats))
        .route("/stats2", post(stats2))
        .route("/tabs", post(tabs))
        .layer(AddExtensionLayer::new(pool))
        .layer(TraceLayer::new_for_http().on_response(DefaultOnResponse::new().level(Level::INFO)));

    let listener = TcpListener::bind(&addr)?;

    let server = Server::from_tcp(listener)?
        .tcp_nodelay(true)
        .serve(app.into_make_service());
    return Ok(server.await?);
}

fn main() -> Result<()> {
    let rt = Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()?;
    rt.block_on(async move { run().await })?;
    Ok(())
}
