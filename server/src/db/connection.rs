use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing::info;

pub async fn create_pool(database_url: &str, max_connections: u32) -> PgPool {
    info!("Connecting to database with pool size: {}", max_connections);
    
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(2)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .max_lifetime(std::time::Duration::from_secs(1800))
        .idle_timeout(std::time::Duration::from_secs(600))
        .connect(database_url)
        .await
        .expect("Failed to create database pool");
    
    info!("Database pool created successfully");
    pool
}

pub async fn run_migrations(pool: &PgPool) {
    info!("Running database migrations...");
    
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .expect("Failed to run migrations");
    
    info!("Migrations completed successfully");
}
