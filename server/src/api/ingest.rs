use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{info, error};

use crate::config::AppState;
use crate::middleware::auth::Claims;
use crate::middleware::error::AppError;
use crate::middleware::permission::check_project_permission;
use crate::services::llm::{LlmConfig, ChatMessage, LlmService};

#[derive(Deserialize)]
pub struct IngestRequest {
    pub content: String,
    pub config: LlmConfig,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct IngestResponse {
    pub task_id: Uuid,
    pub status: String,
}

#[derive(Serialize)]
pub struct IngestStatusResponse {
    pub task_id: Uuid,
    pub status: String,
    pub progress: Option<f64>,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/ingest", post(ingest_content))
        .route("/ingest/:task_id", get(get_ingest_status))
        .route("/ingest/:task_id/cancel", post(cancel_ingest))
        .route("/queue", get(list_ingest_queue))
}

pub async fn ingest_content(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Json(req): Json<IngestRequest>,
) -> Result<(StatusCode, Json<IngestResponse>), AppError> {
    let task_id = Uuid::new_v4();

    info!("Starting ingest task {} for user {}", task_id, claims.0.sub);

    sqlx::query!(
        r#"
        INSERT INTO ingest_tasks (id, user_id, status, content, metadata)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        task_id,
        claims.0.sub,
        "pending",
        req.content,
        req.metadata.as_ref().map(|m| m.to_string()),
    )
    .execute(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to create ingest task: {}", e);
        AppError::Internal
    })?;

    let config_clone = req.config.clone();
    let content_clone = req.content.clone();
    let db_clone = state.db.clone();
    let task_id_clone = task_id;

    tokio::spawn(async move {
        run_ingest_task(task_id_clone, config_clone, content_clone, db_clone).await;
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(IngestResponse {
            task_id,
            status: "pending".to_string(),
        }),
    ))
}

async fn run_ingest_task(
    task_id: Uuid,
    config: LlmConfig,
    content: String,
    db: sqlx::PgPool,
) {
    info!("Processing ingest task {}", task_id);

    if let Err(e) = sqlx::query!(
        "UPDATE ingest_tasks SET status = $1, started_at = NOW() WHERE id = $2",
        "processing",
        task_id,
    )
    .execute(&db)
    .await {
        error!("Failed to update task status: {}", e);
        let _ = sqlx::query!(
            "UPDATE ingest_tasks SET status = $1, error = $2, completed_at = NOW() WHERE id = $3",
            "failed",
            format!("Failed to update task status: {}", e),
            task_id,
        )
        .execute(&db)
        .await;
        return;
    }

    let llm_service = LlmService::new();

    if let Err(e) = sqlx::query!(
        "UPDATE ingest_tasks SET status = $1, progress = $2 WHERE id = $3",
        "processing",
        0.33f64,
        task_id,
    )
    .execute(&db)
    .await {
        error!("Failed to update task progress: {}", e);
        let _ = sqlx::query!(
            "UPDATE ingest_tasks SET status = $1, error = $2, completed_at = NOW() WHERE id = $3",
            "failed",
            format!("Failed to update task progress: {}", e),
            task_id,
        )
        .execute(&db)
        .await;
        return;
    }

    let step1_messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: "You are a knowledge extraction assistant. Analyze the following content and extract key concepts, entities, and their relationships. Output in JSON format with fields: concepts, entities, relationships.".to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: content.clone(),
        },
    ];

    let chat_req = crate::services::llm::ChatRequest {
        messages: step1_messages,
        temperature: Some(0.7),
        max_tokens: Some(4096),
    };

    let step1_result = match llm_service.stream_chat(&config, &chat_req).await {
        Ok(response) => {
            let bytes = response.bytes().await.unwrap_or_default();
            String::from_utf8_lossy(&bytes).to_string()
        }
        Err(e) => {
            error!("Step 1 failed for task {}: {}", task_id, e);
            let _ = sqlx::query!(
                "UPDATE ingest_tasks SET status = $1, error = $2, completed_at = NOW() WHERE id = $3",
                "failed",
                format!("Step 1 failed: {}", e),
                task_id,
            )
            .execute(&db)
            .await;
            return;
        }
    };

    if let Err(e) = sqlx::query!(
        "UPDATE ingest_tasks SET status = $1, progress = $2, step1_result = $3 WHERE id = $4",
        "processing",
        0.66f64,
        step1_result,
        task_id,
    )
    .execute(&db)
    .await {
        error!("Failed to update task after step 1: {}", e);
        let _ = sqlx::query!(
            "UPDATE ingest_tasks SET status = $1, error = $2, completed_at = NOW() WHERE id = $3",
            "failed",
            format!("Failed to update task after step 1: {}", e),
            task_id,
        )
        .execute(&db)
        .await;
        return;
    }

    let step2_messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: "Based on the extracted knowledge, generate wiki-style markdown content with proper structure, links, and references.".to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: format!("Original content:\n{}\n\nExtracted knowledge:\n{}\n\nGenerate wiki markdown:", content, step1_result),
        },
    ];

    let chat_req2 = crate::services::llm::ChatRequest {
        messages: step2_messages,
        temperature: Some(0.7),
        max_tokens: Some(4096),
    };

    let step2_result = match llm_service.stream_chat(&config, &chat_req2).await {
        Ok(response) => {
            let bytes = response.bytes().await.unwrap_or_default();
            String::from_utf8_lossy(&bytes).to_string()
        }
        Err(e) => {
            error!("Step 2 failed for task {}: {}", task_id, e);
            let _ = sqlx::query!(
                "UPDATE ingest_tasks SET status = $1, error = $2, completed_at = NOW() WHERE id = $3",
                "failed",
                format!("Step 2 failed: {}", e),
                task_id,
            )
            .execute(&db)
            .await;
            return;
        }
    };

    if let Err(e) = sqlx::query!(
        "UPDATE ingest_tasks SET status = $1, progress = $2, result = $3, completed_at = NOW() WHERE id = $4",
        "completed",
        1.0f64,
        step2_result,
        task_id,
    )
    .execute(&db)
    .await {
        error!("Failed to complete task: {}", e);
        let _ = sqlx::query!(
            "UPDATE ingest_tasks SET status = $1, error = $2, completed_at = NOW() WHERE id = $3",
            "failed",
            format!("Failed to complete task: {}", e),
            task_id,
        )
        .execute(&db)
        .await;
    }

    info!("Completed ingest task {}", task_id);
}

pub async fn get_ingest_status(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<IngestStatusResponse>, AppError> {
    let task = sqlx::query!(
        r#"
        SELECT id, status, progress, result, error
        FROM ingest_tasks
        WHERE id = $1 AND user_id = $2
        "#,
        task_id,
        claims.0.sub,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to query ingest task: {}", e);
        AppError::Internal
    })?;

    match task {
        Some(t) => Ok(Json(IngestStatusResponse {
            task_id: t.id,
            status: t.status,
            progress: t.progress,
            result: t.result.map(|r| serde_json::Value::String(r)),
            error: t.error,
        })),
        None => Err(AppError::NotFound("Task not found".to_string())),
    }
}

pub async fn cancel_ingest(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Path(task_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query!(
        r#"
        UPDATE ingest_tasks 
        SET status = 'cancelled', completed_at = NOW()
        WHERE id = $1 AND user_id = $2 AND status IN ('pending', 'processing')
        "#,
        task_id,
        claims.0.sub,
    )
    .execute(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to cancel task: {}", e);
        AppError::Internal
    })?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Task not found or cannot be cancelled".to_string()));
    }

    Ok(StatusCode::OK)
}

pub async fn list_ingest_queue(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
) -> Result<Json<Vec<IngestStatusResponse>>, AppError> {
    let tasks = sqlx::query!(
        r#"
        SELECT id, status, progress, result, error
        FROM ingest_tasks
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT 50
        "#,
        claims.0.sub,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to list ingest tasks: {}", e);
        AppError::Internal
    })?;

    Ok(Json(
        tasks
            .into_iter()
            .map(|t| IngestStatusResponse {
                task_id: t.id,
                status: t.status,
                progress: t.progress,
                result: t.result.map(|r| serde_json::Value::String(r)),
                error: t.error,
            })
            .collect(),
    ))
}
