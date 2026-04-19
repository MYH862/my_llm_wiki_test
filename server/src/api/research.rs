use axum::{
    extract::{State, Extension},
    routing::{post, get},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use crate::config::AppState;
use crate::middleware::auth::Claims;
use crate::middleware::error::AppError;
use crate::services::search::{WebSearchService, WebSearchResult};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/search", post(web_search))
        .route("/tasks", get(list_tasks))
        .route("/tasks", post(create_task))
}

#[derive(Debug, Deserialize)]
pub struct WebSearchRequest {
    pub api_key: String,
    pub query: String,
    pub max_results: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct WebSearchResponse {
    pub results: Vec<WebSearchResultItem>,
}

#[derive(Debug, Serialize)]
pub struct WebSearchResultItem {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub source: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub topic: String,
    pub search_queries: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub task_id: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct TaskListResponse {
    pub tasks: Vec<TaskInfo>,
}

#[derive(Debug, Serialize)]
pub struct TaskInfo {
    pub id: String,
    pub topic: String,
    pub status: String,
}

async fn web_search(
    State(_state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Json(request): Json<WebSearchRequest>,
) -> Result<Json<WebSearchResponse>, AppError> {
    let search_service = WebSearchService::new();
    let max_results = request.max_results.unwrap_or(10);

    let results = search_service
        .tavily_search(&request.api_key, &request.query, max_results)
        .await
        .map_err(|e| AppError::Internal)?;

    Ok(Json(WebSearchResponse {
        results: results.into_iter().map(|r| WebSearchResultItem {
            title: r.title,
            url: r.url,
            snippet: r.snippet,
            source: r.source,
        }).collect(),
    }))
}

async fn list_tasks(
    State(_state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> Result<Json<TaskListResponse>, AppError> {
    Ok(Json(TaskListResponse {
        tasks: vec![],
    }))
}

async fn create_task(
    State(_state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Json(_request): Json<CreateTaskRequest>,
) -> Result<Json<TaskResponse>, AppError> {
    let task_id = uuid::Uuid::new_v4().to_string();

    Ok(Json(TaskResponse {
        task_id,
        status: "queued".to_string(),
    }))
}
