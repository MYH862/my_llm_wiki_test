use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, put, delete},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{info, error};

use crate::config::AppState;
use crate::middleware::auth::Claims;
use crate::middleware::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfigResponse {
    pub id: Uuid,
    pub name: String,
    pub provider: String,
    pub model: String,
    pub has_api_key: bool,
    pub ollama_url: Option<String>,
    pub custom_endpoint: Option<String>,
    pub is_default: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
pub struct CreateLlmConfigRequest {
    pub name: String,
    pub provider: String,
    pub api_key: Option<String>,
    pub model: String,
    pub ollama_url: Option<String>,
    pub custom_endpoint: Option<String>,
    pub is_default: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateLlmConfigRequest {
    pub name: Option<String>,
    pub provider: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub ollama_url: Option<String>,
    pub custom_endpoint: Option<String>,
    pub is_default: Option<bool>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/configs", get(list_configs))
        .route("/configs", post(create_config))
        .route("/configs/:id", get(get_config))
        .route("/configs/:id", put(update_config))
        .route("/configs/:id", delete(delete_config))
        .route("/configs/:id/default", post(set_default_config))
}

pub async fn list_configs(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
) -> Result<Json<Vec<LlmConfigResponse>>, AppError> {
    let configs = sqlx::query!(
        r#"
        SELECT id, name, provider, model, api_key IS NOT NULL as has_api_key,
               ollama_url, custom_endpoint, is_default, created_at
        FROM llm_configs
        WHERE user_id = $1
        ORDER BY is_default DESC, created_at DESC
        "#,
        claims.0.sub,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to list LLM configs: {}", e);
        AppError::Internal
    })?;

    Ok(Json(
        configs
            .into_iter()
            .map(|c| LlmConfigResponse {
                id: c.id,
                name: c.name,
                provider: c.provider,
                model: c.model,
                has_api_key: c.has_api_key.unwrap_or(false),
                ollama_url: c.ollama_url,
                custom_endpoint: c.custom_endpoint,
                is_default: c.is_default,
                created_at: c.created_at,
            })
            .collect(),
    ))
}

pub async fn create_config(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Json(req): Json<CreateLlmConfigRequest>,
) -> Result<(StatusCode, Json<LlmConfigResponse>), AppError> {
    let is_default = req.is_default.unwrap_or(false);

    if is_default {
        sqlx::query!(
            "UPDATE llm_configs SET is_default = false WHERE user_id = $1",
            claims.0.sub,
        )
        .execute(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to clear default configs: {}", e);
            AppError::Internal
        })?;
    }

    let config = sqlx::query!(
        r#"
        INSERT INTO llm_configs (user_id, name, provider, api_key, model, ollama_url, custom_endpoint, is_default)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, name, provider, model, api_key IS NOT NULL as has_api_key,
                  ollama_url, custom_endpoint, is_default, created_at
        "#,
        claims.0.sub,
        req.name,
        req.provider,
        req.api_key,
        req.model,
        req.ollama_url,
        req.custom_endpoint,
        is_default,
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to create LLM config: {}", e);
        AppError::Internal
    })?;

    info!("Created LLM config {} for user {}", config.id, claims.0.sub);

    Ok((
        StatusCode::CREATED,
        Json(LlmConfigResponse {
            id: config.id,
            name: config.name,
            provider: config.provider,
            model: config.model,
            has_api_key: config.has_api_key.unwrap_or(false),
            ollama_url: config.ollama_url,
            custom_endpoint: config.custom_endpoint,
            is_default: config.is_default,
            created_at: config.created_at,
        }),
    ))
}

pub async fn get_config(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<LlmConfigResponse>, AppError> {
    let config = sqlx::query!(
        r#"
        SELECT id, name, provider, model, api_key IS NOT NULL as has_api_key,
               ollama_url, custom_endpoint, is_default, created_at
        FROM llm_configs
        WHERE id = $1 AND user_id = $2
        "#,
        id,
        claims.0.sub,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to get LLM config: {}", e);
        AppError::Internal
    })?;

    match config {
        Some(c) => Ok(Json(LlmConfigResponse {
            id: c.id,
            name: c.name,
            provider: c.provider,
            model: c.model,
            has_api_key: c.has_api_key.unwrap_or(false),
            ollama_url: c.ollama_url,
            custom_endpoint: c.custom_endpoint,
            is_default: c.is_default,
            created_at: c.created_at,
        })),
        None => Err(AppError::NotFound("LLM config not found".to_string())),
    }
}

pub async fn update_config(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateLlmConfigRequest>,
) -> Result<Json<LlmConfigResponse>, AppError> {
    if let Some(true) = req.is_default {
        sqlx::query!(
            "UPDATE llm_configs SET is_default = false WHERE user_id = $1 AND id != $2",
            claims.0.sub,
            id,
        )
        .execute(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to clear default configs: {}", e);
            AppError::Internal
        })?;
    }

    let config = sqlx::query!(
        r#"
        UPDATE llm_configs
        SET name = COALESCE($1, name),
            provider = COALESCE($2, provider),
            api_key = COALESCE($3, api_key),
            model = COALESCE($4, model),
            ollama_url = COALESCE($5, ollama_url),
            custom_endpoint = COALESCE($6, custom_endpoint),
            is_default = COALESCE($7, is_default),
            updated_at = NOW()
        WHERE id = $8 AND user_id = $9
        RETURNING id, name, provider, model, api_key IS NOT NULL as has_api_key,
                  ollama_url, custom_endpoint, is_default, created_at
        "#,
        req.name,
        req.provider,
        req.api_key,
        req.model,
        req.ollama_url,
        req.custom_endpoint,
        req.is_default,
        id,
        claims.0.sub,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to update LLM config: {}", e);
        AppError::Internal
    })?;

    match config {
        Some(c) => Ok(Json(LlmConfigResponse {
            id: c.id,
            name: c.name,
            provider: c.provider,
            model: c.model,
            has_api_key: c.has_api_key.unwrap_or(false),
            ollama_url: c.ollama_url,
            custom_endpoint: c.custom_endpoint,
            is_default: c.is_default,
            created_at: c.created_at,
        })),
        None => Err(AppError::NotFound("LLM config not found".to_string())),
    }
}

pub async fn delete_config(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query!(
        "DELETE FROM llm_configs WHERE id = $1 AND user_id = $2",
        id,
        claims.0.sub,
    )
    .execute(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to delete LLM config: {}", e);
        AppError::Internal
    })?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("LLM config not found".to_string()));
    }

    info!("Deleted LLM config {} for user {}", id, claims.0.sub);
    Ok(StatusCode::NO_CONTENT)
}

pub async fn set_default_config(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<LlmConfigResponse>, AppError> {
    sqlx::query!(
        "UPDATE llm_configs SET is_default = false WHERE user_id = $1",
        claims.0.sub,
    )
    .execute(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to clear default configs: {}", e);
        AppError::Internal
    })?;

    let config = sqlx::query!(
        r#"
        UPDATE llm_configs
        SET is_default = true, updated_at = NOW()
        WHERE id = $1 AND user_id = $2
        RETURNING id, name, provider, model, api_key IS NOT NULL as has_api_key,
                  ollama_url, custom_endpoint, is_default, created_at
        "#,
        id,
        claims.0.sub,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to set default config: {}", e);
        AppError::Internal
    })?;

    match config {
        Some(c) => Ok(Json(LlmConfigResponse {
            id: c.id,
            name: c.name,
            provider: c.provider,
            model: c.model,
            has_api_key: c.has_api_key.unwrap_or(false),
            ollama_url: c.ollama_url,
            custom_endpoint: c.custom_endpoint,
            is_default: c.is_default,
            created_at: c.created_at,
        })),
        None => Err(AppError::NotFound("LLM config not found".to_string())),
    }
}
