use llm_wiki_server::{create_app, create_state, config::Config};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() {
    let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into());
    
    tracing_subscriber::registry()
        .with(EnvFilter::new(&log_level))
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .with_level(true)
        )
        .init();

    let config = Config::from_env();
    let addr = format!("{}:{}", config.server.host, config.server.port);

    info!("Starting LLM Wiki Server on {}", addr);
    info!("Database URL: {}", config.database.url);
    info!("MinIO Endpoint: {}", config.minio.endpoint);
    info!("Qdrant URL: {}", config.qdrant.url);
    info!("CORS Origins: {}", config.cors.allowed_origins);
    info!("Log Level: {}", log_level);

    let state = create_state(config).await;
    let app = create_app(state).await;

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    info!("Server listening on {}", addr);
    info!("Open http://{}/health to check server status", addr);

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
