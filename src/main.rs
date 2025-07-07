use axum::{routing::get, Router};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::{env, net::SocketAddr};
use tracing_subscriber;
use anyhow::Result;

mod routes;
mod models;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let app = Router::new()
        .merge(routes::cycle::routes(pool.clone()))
        .merge(routes::symptoms::routes(pool.clone()))
        .merge(routes::bleeding::routes(pool.clone()))
        .merge(routes::cycle_stats::routes(pool.clone()))
        .route("/health", get(|| async { "✅ Backend up" }));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3050));
    tracing::info!("🧠 Server running at {}", addr);

    axum::serve(
        tokio::net::TcpListener::bind(addr).await?,
        app.into_make_service(),
    )
    .await?;

    Ok(())
}
