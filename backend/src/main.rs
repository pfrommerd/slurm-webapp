use anyhow::{Context, Result};
use axum::{extract::State, routing::get, Json, Router};
use chrono::Utc;
use env_logger::Env;
use log::info;
use slurm_common::{ClusterStatus, Job, JobState, Node, Partition};
use sqlx::{sqlite::SqlitePoolOptions, FromRow, Pool, Sqlite};
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
        .route("/api/status", get(get_status))
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

async fn get_status(State(state): State<AppState>) -> Json<ClusterStatus> {
    // In a real optimized app we might Cache this or query tables separately.
    // Reconstructing ClusterStatus from DB.
    let nodes = fetch_nodes(&state.pool).await.unwrap_or_default();
    let jobs = fetch_jobs(&state.pool).await.unwrap_or_default();
    let partitions = fetch_partitions(&state.pool).await.unwrap_or_default();

    // updated_at is roughly max of updated_at in tables, or just now for simplicity as this is an aggregate view
    let updated_at = Utc::now();

    Json(ClusterStatus {
        nodes,
        jobs,
        partitions,
        updated_at,
    })
}

async fn get_nodes(State(state): State<AppState>) -> Json<Vec<Node>> {
    let nodes = fetch_nodes(&state.pool).await.unwrap_or(vec![]);
    Json(nodes)
}

async fn get_jobs(State(state): State<AppState>) -> Json<Vec<Job>> {
    let jobs = fetch_jobs(&state.pool).await.unwrap_or(vec![]);
    Json(jobs)
}

async fn get_partitions(State(state): State<AppState>) -> Json<Vec<Partition>> {
    let parts = fetch_partitions(&state.pool).await.unwrap_or(vec![]);
    Json(parts)
}

// Helpers

#[derive(FromRow)]
struct NodeRow {
    name: String,
    state: String,
    cpus: i64,
    real_memory: i64,
    resources: Option<String>,
}

async fn fetch_nodes(pool: &Pool<Sqlite>) -> Result<Vec<Node>> {
    let rows = sqlx::query_as::<_, NodeRow>("SELECT * FROM nodes ORDER BY name")
        .fetch_all(pool)
        .await?;

    let nodes = rows
        .into_iter()
        .map(|row| Node {
            name: row.name,
            state: row.state,
            cpus: row.cpus as u32,
            real_memory: row.real_memory,
            resources: serde_json::from_str(&row.resources.unwrap_or_default()).unwrap_or_default(),
        })
        .collect();
    Ok(nodes)
}

#[derive(FromRow)]
struct JobRow {
    job_id: String,
    user: String,
    partition: String,
    state: String,
    num_nodes: i64,
    num_cpus: i64,
    time_limit: Option<String>,
    start_time: Option<chrono::DateTime<chrono::Utc>>,
    submit_time: chrono::DateTime<chrono::Utc>,
}

async fn fetch_jobs(pool: &Pool<Sqlite>) -> Result<Vec<Job>> {
    let rows = sqlx::query_as::<_, JobRow>("SELECT * FROM jobs ORDER BY submit_time DESC")
        .fetch_all(pool)
        .await?;

    let jobs = rows
        .into_iter()
        .map(|row| {
            // Parse state string back to Enum. If invalid, default to UNKNOWN or handle error.
            let state_enum = match row.state.as_str() {
                "PENDING" => JobState::PENDING,
                "RUNNING" => JobState::RUNNING,
                "COMPLETED" => JobState::COMPLETED,
                "FAILED" => JobState::FAILED,
                "CANCELLED" => JobState::CANCELLED,
                _ => JobState::UNKNOWN,
            };

            Job {
                job_id: row.job_id,
                user: row.user,
                partition: row.partition,
                state: state_enum,
                num_nodes: row.num_nodes as u32,
                num_cpus: row.num_cpus as u32,
                time_limit: row.time_limit,
                start_time: row.start_time,
                submit_time: row.submit_time,
            }
        })
        .collect();

    Ok(jobs)
}

#[derive(FromRow)]
struct PartitionRow {
    name: String,
    total_nodes: i64,
    total_cpus: i64,
    state: String,
}

async fn fetch_partitions(pool: &Pool<Sqlite>) -> Result<Vec<Partition>> {
    let rows = sqlx::query_as::<_, PartitionRow>("SELECT * FROM partitions ORDER BY name")
        .fetch_all(pool)
        .await?;

    let parts = rows
        .into_iter()
        .map(|row| Partition {
            name: row.name,
            total_nodes: row.total_nodes as u32,
            total_cpus: row.total_cpus as u32,
            state: row.state,
        })
        .collect();

    Ok(parts)
}
