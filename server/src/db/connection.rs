use sqlx::PgPool;
use tracing::info;

pub async fn create_pool(database_url: &str, max_connections: u32) -> PgPool {
    info!("Connecting to database...");
    
    PgPool::connect(database_url)
        .await
        .expect("Failed to create database pool")
}

pub async fn run_migrations(pool: &PgPool) {
    info!("Running database migrations...");
    
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .expect("Failed to run migrations");
    
    info!("Migrations completed successfully");
}
