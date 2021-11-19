use std::net::{IpAddr, SocketAddr, TcpListener};

use anyhow::{Context, Result};
use axum::{extract::Extension, routing::post, AddExtensionLayer, Json, Router, Server};
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod};
use tokio::runtime::Builder;
use tokio_postgres::NoTls;
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tracing::Level;

use self::{
    authentication::Authentication,
    models::{Stats, Stats2, Tabs},
};

mod authentication;
mod models;

async fn stats(
    _authorization: Authentication,
    pool: Extension<Pool>,
    stats: Json<Stats>,
) -> Result<(), String> {
    let conn = pool.get().await.unwrap();
    match conn
        .execute(
            "call insert_stats($1, $2, $3)",
            &[&stats.time, &stats.temperature, &stats.humidity],
        )
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

async fn stats2(
    _authorization: Authentication,
    pool: Extension<Pool>,
    stats: Json<Stats2>,
) -> Result<(), String> {
    let conn = pool.get().await.unwrap();
    match conn
        .execute(
            "call insert_stats2($1, $2, $3)",
            &[&stats.time, &stats.temperature, &(stats.co2 as i16)],
        )
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

async fn tabs(
    _authorization: Authentication,
    pool: Extension<Pool>,
    tabs: Json<Tabs>,
) -> Result<(), String> {
    let conn = pool.get().await.unwrap();
    match conn
        .execute(
            "insert into tabs(time, tabs) values($1, $2)",
            &[&tabs.time, &(tabs.tabs as i16)],
        )
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

async fn run() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "stats_logger=info,tower_http=info")
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
    cfg.dbname = Some("room_stats".to_string());
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    let pool = cfg
        .create_pool(Some(deadpool_postgres::Runtime::Tokio1), NoTls)
        .unwrap();

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
