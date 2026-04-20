mod api;
pub mod config;
mod db;
mod middleware;
mod models;
mod services;
mod utils;

#[cfg(test)]
mod tests;

use axum::{routing::get, Router};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tower_http::limit::RequestBodyLimitLayer;

use crate::config::{AppState, AppStateInner, Config};
use crate::services::file::MinIOService;
use crate::services::vector::QdrantService;
use crate::middleware::rate_limit::create_rate_limiter;

pub async fn create_app(state: AppState) -> Router {
    let cors_origins = state.config.cors_origins();
    
    let cors = if cors_origins.iter().any(|o| o == "*") {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        CorsLayer::new()
            .allow_origin(
                cors_origins
                    .into_iter()
                    .map(|o| o.parse().unwrap())
                    .collect::<Vec<_>>(),
            )
            .allow_methods(Any)
            .allow_headers(Any)
    };

    let rate_limiter = create_rate_limiter(10);

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
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024))
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

    let qdrant = QdrantService::new(
        &config.qdrant.url,
        &config.qdrant.api_key,
        &config.qdrant.collection_prefix,
        config.qdrant.vector_size,
    )
    .expect("Failed to initialize Qdrant service");

    std::sync::Arc::new(AppStateInner { config, db, minio, qdrant })
}
