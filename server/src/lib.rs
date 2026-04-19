mod api;
mod config;
mod db;
mod middleware;
mod models;
mod services;
mod utils;

use axum::{routing::get, Router};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::config::{AppState, AppStateInner, Config};
use crate::services::file::MinIOService;

pub async fn create_app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(
            state
                .config
                .cors_origins()
                .into_iter()
                .map(|o| o.parse().unwrap())
                .collect::<Vec<_>>(),
        )
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/api/auth", api::auth::router())
        .nest("/api/users", api::users::router())
        .nest("/api/projects", api::projects::router())
        .nest("/api/files", api::files::project_router())
        .nest("/api/chat", api::chat::router())
        .nest("/api/ingest", api::ingest::router())
        .nest("/api/llm", api::llm_configs::router())
        .nest("/api/graph", api::graph::router())
        .nest("/api/search", api::search::router())
        .nest("/api/research", api::research::router())
        .nest("/api/review", api::review::router())
        .nest("/api/lint", api::lint::router())
        .nest("/api/vector", api::vector::router())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    app
}

async fn health_check() -> &'static str {
    "OK"
}

pub async fn create_state(config: Config) -> AppState {
    let db = db::connection::create_pool(&config.database.url, config.database.max_connections).await;
    db::connection::run_migrations(&db).await;

    let minio = MinIOService::new(
        &config.minio.endpoint,
        &config.minio.access_key,
        &config.minio.secret_key,
        config.minio.use_ssl,
        &config.minio.bucket_prefix,
    )
    .expect("Failed to initialize MinIO service");

    std::sync::Arc::new(AppStateInner { config, db, minio })
}
