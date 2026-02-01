use anyhow::{Context, Result};
use axum::{extract::State, routing::get, Json, Router};
use env_logger::Env;
use log::info;
use slurm_common::{db, ClusterState};

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
        .route("/api/state", get(get_state))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Backend listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn get_state(State(state): State<AppState>) -> Json<ClusterState> {
    let raw_state = db::fetch_cluster_state(&state.pool)
        .await
        .unwrap_or_default();
    Json(raw_state)
}
