use axum::{
    extract::{State, Extension, Path},
    routing::{post, get, delete},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use crate::config::AppState;
use crate::middleware::auth::Claims;
use crate::middleware::error::AppError;
use crate::services::vector::{VectorUpsertRequest, VectorSearchRequest};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/upsert", post(upsert_vector))
        .route("/search", post(search_vectors))
        .route("/delete", delete(delete_vector))
        .route("/count/:project_id", get(count_vectors))
}

#[derive(Debug, Deserialize)]
pub struct UpsertRequest {
    pub project_id: String,
    pub page_id: String,
    pub embedding: Vec<f32>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub project_id: String,
    pub query_embedding: Vec<f32>,
    pub top_k: usize,
    pub filter_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteRequest {
    pub project_id: String,
    pub page_id: String,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResultItem>,
}

#[derive(Debug, Serialize)]
pub struct SearchResultItem {
    pub page_id: String,
    pub score: f32,
}

#[derive(Debug, Serialize)]
pub struct CountResponse {
    pub count: usize,
}

async fn upsert_vector(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Json(request): Json<UpsertRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let upsert_request = VectorUpsertRequest {
        project_id: request.project_id.clone(),
        page_id: request.page_id.clone(),
        embedding: request.embedding,
        metadata: request.metadata.map(|m| {
            if let serde_json::Value::Object(map) = m {
                map.into_iter().collect()
            } else {
                std::collections::HashMap::new()
            }
        }),
    };
    
    state.qdrant.upsert(upsert_request)
        .await
        .map_err(|e| AppError::Internal)?;
    
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": format!("Vector upserted for page: {}", request.page_id)
    })))
}

async fn search_vectors(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, AppError> {
    let search_request = VectorSearchRequest {
        project_id: request.project_id.clone(),
        query_embedding: request.query_embedding,
        top_k: request.top_k,
        filter_metadata: request.filter_metadata.map(|m| {
            if let serde_json::Value::Object(map) = m {
                map.into_iter().collect()
            } else {
                std::collections::HashMap::new()
            }
        }),
    };
    
    let results = state.qdrant.search(search_request)
        .await
        .map_err(|e| AppError::Internal)?;
    
    Ok(Json(SearchResponse {
        results: results.into_iter().map(|r| SearchResultItem {
            page_id: r.page_id,
            score: r.score,
        }).collect(),
    }))
}

async fn delete_vector(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Json(request): Json<DeleteRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    state.qdrant.delete(&request.project_id, &request.page_id)
        .await
        .map_err(|e| AppError::Internal)?;
    
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": format!("Vector deleted for page: {}", request.page_id)
    })))
}

async fn count_vectors(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(project_id): Path<String>,
) -> Result<Json<CountResponse>, AppError> {
    let count = state.qdrant.count(&project_id)
        .await
        .map_err(|e| AppError::Internal)?;
    
    Ok(Json(CountResponse { count }))
}
