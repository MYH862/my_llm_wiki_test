use axum::{
    extract::{State, Path, Extension},
    routing::{post, get, put, delete},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::config::AppState;
use crate::middleware::auth::Claims;
use crate::middleware::error::AppError;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_reviews))
        .route("/", post(create_review))
        .route("/:id", get(get_review))
        .route("/:id", put(update_review))
        .route("/:id", delete(delete_review))
}

#[derive(Debug, Deserialize)]
pub struct CreateReviewRequest {
    pub content_id: String,
    pub content_type: String,
    pub reviewer_id: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateReviewRequest {
    pub status: Option<String>,
    pub reviewer_id: Option<String>,
    pub notes: Option<String>,
    pub feedback: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReviewResponse {
    pub id: String,
    pub content_id: String,
    pub content_type: String,
    pub status: String,
    pub reviewer_id: Option<String>,
    pub notes: Option<String>,
    pub feedback: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ReviewListResponse {
    pub reviews: Vec<ReviewResponse>,
}

async fn list_reviews(
    State(_state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> Result<Json<ReviewListResponse>, AppError> {
    Ok(Json(ReviewListResponse {
        reviews: vec![],
    }))
}

async fn create_review(
    State(_state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Json(request): Json<CreateReviewRequest>,
) -> Result<Json<ReviewResponse>, AppError> {
    let now = Utc::now();
    let id = Uuid::new_v4().to_string();

    Ok(Json(ReviewResponse {
        id,
        content_id: request.content_id,
        content_type: request.content_type,
        status: "pending".to_string(),
        reviewer_id: request.reviewer_id,
        notes: request.notes,
        feedback: None,
        created_at: now,
        updated_at: now,
    }))
}

async fn get_review(
    State(_state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(_id): Path<String>,
) -> Result<Json<ReviewResponse>, AppError> {
    Err(AppError::NotFound("Review not found".to_string()))
}

async fn update_review(
    State(_state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(request): Json<UpdateReviewRequest>,
) -> Result<Json<ReviewResponse>, AppError> {
    let now = Utc::now();

    Ok(Json(ReviewResponse {
        id,
        content_id: "unknown".to_string(),
        content_type: "unknown".to_string(),
        status: request.status.unwrap_or_else(|| "pending".to_string()),
        reviewer_id: request.reviewer_id,
        notes: request.notes,
        feedback: request.feedback,
        created_at: now,
        updated_at: now,
    }))
}

async fn delete_review(
    State(_state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Review deleted"
    })))
}
