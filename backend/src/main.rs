use anyhow::{Context, Result};
use async_graphql::http::GraphiQLSource;
use async_graphql_axum::GraphQL;
use axum::response::{Html, IntoResponse};
use axum::{routing::get, Router};
use backend::api::schema;
use env_logger::Env;
use log::info;

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
        .route(
            "/api/graphql",
            get(graphiql).post_service(GraphQL::new(schema())),
        )
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Backend listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/api/graphql").finish())
}
