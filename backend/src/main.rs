mod app;
mod config;
mod error;
mod models;
mod state;

use anyhow::Context;
use config::Config;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .context("failed to connect to postgres")?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("failed to run database migrations")?;

    let state = state::AppState::new(pool);
    let app = app::build_router(state, &config.app_cors_origin);
    let listener = TcpListener::bind((config.app_host.as_str(), config.app_port))
        .await
        .context("failed to bind TCP listener")?;

    tracing::info!(
        "listening on http://{}:{}",
        config.app_host,
        config.app_port
    );
    axum::serve(listener, app).await.context("server error")?;
    Ok(())
}
