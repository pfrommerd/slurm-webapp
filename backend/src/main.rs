use anyhow::{Context, Result};
use axum::{extract::State, routing::get, Json, Router};
use chrono::{DateTime, Utc};
use env_logger::Env;
use log::info;
use slurm_common::{db, Job, Node, Partition};

use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
struct AppState {
    pool: Pool<Sqlite>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    dotenv::dotenv().ok();

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&std::env::var("DATABASE_URL")?)
        .await
        .context("Failed to connect to database in backend")?;

    // Run migrations
    sqlx::migrate!("../migrations")
        .run(&pool)
        .await
        .context("Failed to run migrations")?;

    let state = AppState { pool };

    let app = Router::new()
        .route("/api/updated_at", get(get_updated_at))
        .route("/api/nodes", get(get_nodes))
        .route("/api/jobs", get(get_jobs))
        .route("/api/partitions", get(get_partitions))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Backend listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn get_updated_at(State(state): State<AppState>) -> Json<Option<DateTime<Utc>>> {
    let updated_at = db::fetch_metadata::<DateTime<Utc>>(&state.pool, "last_updated")
        .await
        .unwrap_or(None);
    Json(updated_at)
}

async fn get_nodes(State(state): State<AppState>) -> Json<Vec<Node>> {
    let nodes = db::fetch_all_nodes(&state.pool).await.unwrap_or(vec![]);
    Json(nodes)
}

async fn get_jobs(State(state): State<AppState>) -> Json<Vec<Job>> {
    let jobs = db::fetch_all_jobs(&state.pool).await.unwrap_or(vec![]);
    Json(jobs)
}

async fn get_partitions(State(state): State<AppState>) -> Json<Vec<Partition>> {
    let parts = db::fetch_all_partitions(&state.pool)
        .await
        .unwrap_or(vec![]);
    Json(parts)
}

// Helpers removed as they are now in slurm-common
