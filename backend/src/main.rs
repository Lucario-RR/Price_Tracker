mod app;
mod config;
mod error;
mod models;
mod state;

use anyhow::Context;
use config::Config;
use sqlx::postgres::PgPoolOptions;
use std::{fs, path::Path};
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

async fn applied_v2_migrations(pool: &sqlx::PgPool) -> anyhow::Result<Vec<String>> {
    let filenames = sqlx::query_scalar::<_, String>(
        r#"
        SELECT filename
        FROM _app_sql_migrations
        WHERE filename LIKE '%\_v2.sql' ESCAPE '\'
        ORDER BY filename
        "#,
    )
    .fetch_all(pool)
    .await
    .context("failed to read applied v2 migrations")?;

    Ok(filenames)
}

async fn run_sql_migrations(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    let migrations_dir = Path::new("./migrations");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS _app_sql_migrations (
            filename TEXT PRIMARY KEY,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await
    .context("failed to create _app_sql_migrations table")?;

    let mut files = fs::read_dir(migrations_dir)
        .with_context(|| format!("failed to read {}", migrations_dir.display()))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_file() {
                return None;
            }

            let file_name = path.file_name()?.to_str()?.to_string();
            if !file_name.ends_with("_v2.sql") {
                return None;
            }

            Some((file_name, path))
        })
        .collect::<Vec<_>>();

    files.sort_by_key(|(file_name, _)| file_name.clone());

    for (file_name, path) in files {
        let already_applied = sqlx::query_scalar::<_, String>(
            "SELECT filename FROM _app_sql_migrations WHERE filename = $1",
        )
        .bind(&file_name)
        .fetch_optional(pool)
        .await
        .with_context(|| format!("failed to check migration state for {file_name}"))?;

        if already_applied.is_some() {
            continue;
        }

        let sql = fs::read_to_string(&path)
            .with_context(|| format!("failed to read migration {}", path.display()))?;

        sqlx::raw_sql(&sql)
            .execute(pool)
            .await
            .with_context(|| format!("failed to execute migration {file_name}"))?;
        sqlx::query("INSERT INTO _app_sql_migrations (filename) VALUES ($1)")
            .bind(&file_name)
            .execute(pool)
            .await
            .with_context(|| format!("failed to record migration {file_name}"))?;

        tracing::info!("applied migration {}", file_name);
    }

    Ok(())
}

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

    run_sql_migrations(&pool)
        .await
        .context("failed to run database migrations")?;

    let applied_migrations = applied_v2_migrations(&pool).await?;
    tracing::info!(
        "confirmed applied _v2.sql scripts: {}",
        if applied_migrations.is_empty() {
            "(none)".to_string()
        } else {
            applied_migrations.join(", ")
        }
    );

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
