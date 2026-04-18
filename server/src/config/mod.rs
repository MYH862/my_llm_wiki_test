use serde::Deserialize;
use std::sync::Arc;
use crate::services::file::MinIOService;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    pub cors: CorsConfig,
    pub llm: LlmConfig,
    pub storage: StorageConfig,
    pub minio: MinIOConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MinIOConfig {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub use_ssl: bool,
    pub bucket_prefix: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration_hours: u64,
    pub refresh_expiration_days: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CorsConfig {
    pub allowed_origins: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlmConfig {
    pub api_key: String,
    pub api_url: String,
    pub model: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    pub file_storage_path: String,
    pub vector_db_path: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            server: ServerConfig {
                host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: std::env::var("SERVER_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(3000),
            },
            database: DatabaseConfig {
                url: std::env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/llm_wiki".to_string()),
                max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                    .ok()
                    .and_then(|n| n.parse().ok())
                    .unwrap_or(10),
            },
            jwt: JwtConfig {
                secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string()),
                expiration_hours: std::env::var("JWT_EXPIRATION_HOURS")
                    .ok()
                    .and_then(|n| n.parse().ok())
                    .unwrap_or(24),
                refresh_expiration_days: std::env::var("JWT_REFRESH_EXPIRATION_DAYS")
                    .ok()
                    .and_then(|n| n.parse().ok())
                    .unwrap_or(7),
            },
            cors: CorsConfig {
                allowed_origins: std::env::var("ALLOWED_ORIGINS")
                    .unwrap_or_else(|_| "http://localhost:5173".to_string()),
            },
            llm: LlmConfig {
                api_key: std::env::var("LLM_API_KEY").unwrap_or_default(),
                api_url: std::env::var("LLM_API_URL")
                    .unwrap_or_else(|_| "https://api.openai.com/v1/chat/completions".to_string()),
                model: std::env::var("LLM_MODEL").unwrap_or_else(|_| "gpt-4".to_string()),
            },
            storage: StorageConfig {
                file_storage_path: std::env::var("FILE_STORAGE_PATH")
                    .unwrap_or_else(|_| "/data/wiki".to_string()),
                vector_db_path: std::env::var("VECTOR_DB_PATH")
                    .unwrap_or_else(|_| "/data/vectors".to_string()),
            },
            minio: MinIOConfig {
                endpoint: std::env::var("MINIO_ENDPOINT")
                    .unwrap_or_else(|_| "localhost:9000".to_string()),
                access_key: std::env::var("MINIO_ACCESS_KEY")
                    .unwrap_or_else(|_| "minioadmin".to_string()),
                secret_key: std::env::var("MINIO_SECRET_KEY")
                    .unwrap_or_else(|_| "minioadmin".to_string()),
                use_ssl: std::env::var("MINIO_USE_SSL")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(false),
                bucket_prefix: std::env::var("MINIO_BUCKET_PREFIX")
                    .unwrap_or_else(|_| "llm-wiki".to_string()),
            },
        }
    }

    pub fn cors_origins(&self) -> Vec<String> {
        self.cors
            .allowed_origins
            .split(',')
            .map(|s| s.trim().to_string())
            .collect()
    }
}

pub type AppState = Arc<AppStateInner>;

pub struct AppStateInner {
    pub config: Config,
    pub db: sqlx::PgPool,
    pub minio: MinIOService,
}
