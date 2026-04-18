use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::AppState;
use crate::middleware::auth::Claims;
use crate::middleware::error::AppError;
use crate::middleware::permission::check_project_permission;
use crate::models::file::{FileInfo, DirectoryEntry, FileContent};

#[derive(Deserialize)]
pub struct ListQuery {
    pub path: Option<String>,
}

#[derive(Deserialize)]
pub struct ReadQuery {
    pub path: String,
}

#[derive(Deserialize)]
pub struct WriteRequest {
    pub path: String,
    pub content: String,
    pub content_type: Option<String>,
}

#[derive(Deserialize)]
pub struct CopyRequest {
    pub from_path: String,
    pub to_path: String,
}

#[derive(Deserialize)]
pub struct DeleteQuery {
    pub path: String,
}

pub async fn list_files(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Path(project_id): Path<Uuid>,
    Query(query): Query<ListQuery>,
) -> Result<Json<DirectoryEntry>, AppError> {
    check_project_permission(
        &state.db,
        claims.0.sub,
        project_id,
        "files:read",
    )
    .await
    .map_err(|_| AppError::PermissionDenied)?;

    let prefix = query.path.unwrap_or_default();
    let files = state
        .minio
        .list_files(&project_id.to_string(), &prefix)
        .await?;

    let entries: Vec<FileInfo> = files
        .iter()
        .map(|path| FileInfo {
            name: path.split('/').last().unwrap_or(path).to_string(),
            path: path.clone(),
            is_directory: false,
            size: 0,
            modified_at: None,
        })
        .collect();

    Ok(Json(DirectoryEntry {
        path: prefix,
        entries,
    }))
}

pub async fn read_file(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Path(project_id): Path<Uuid>,
    Query(query): Query<ReadQuery>,
) -> Result<Json<FileContent>, AppError> {
    check_project_permission(
        &state.db,
        claims.0.sub,
        project_id,
        "files:read",
    )
    .await
    .map_err(|_| AppError::PermissionDenied)?;

    let content = state.minio.download_file(&project_id.to_string(), &query.path).await?;
    let content_str = String::from_utf8(content).map_err(|e| AppError::BadRequest(e.to_string()))?;

    let content_type = if query.path.ends_with(".md") {
        "text/markdown"
    } else if query.path.ends_with(".json") {
        "application/json"
    } else {
        "text/plain"
    }
    .to_string();

    Ok(Json(FileContent {
        path: query.path,
        content: content_str,
        content_type,
    }))
}

pub async fn write_file(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<WriteRequest>,
) -> Result<StatusCode, AppError> {
    check_project_permission(
        &state.db,
        claims.0.sub,
        project_id,
        "files:write",
    )
    .await
    .map_err(|_| AppError::PermissionDenied)?;

    let content_type = req.content_type.unwrap_or_else(|| {
        if req.path.ends_with(".md") {
            "text/markdown"
        } else {
            "text/plain"
        }
        .to_string()
    });

    state
        .minio
        .upload_file(
            &project_id.to_string(),
            &req.path,
            req.content.as_bytes(),
            &content_type,
        )
        .await?;

    Ok(StatusCode::OK)
}

pub async fn delete_file(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Path(project_id): Path<Uuid>,
    Query(query): Query<DeleteQuery>,
) -> Result<StatusCode, AppError> {
    check_project_permission(
        &state.db,
        claims.0.sub,
        project_id,
        "files:delete",
    )
    .await
    .map_err(|_| AppError::PermissionDenied)?;

    state
        .minio
        .delete_file(&project_id.to_string(), &query.path)
        .await?;

    Ok(StatusCode::OK)
}

pub async fn copy_file(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<CopyRequest>,
) -> Result<StatusCode, AppError> {
    check_project_permission(
        &state.db,
        claims.0.sub,
        project_id,
        "files:write",
    )
    .await
    .map_err(|_| AppError::PermissionDenied)?;

    let content = state
        .minio
        .download_file(&project_id.to_string(), &req.from_path)
        .await?;

    let content_type = if req.to_path.ends_with(".md") {
        "text/markdown"
    } else {
        "text/plain"
    }
    .to_string();

    state
        .minio
        .upload_file(
            &project_id.to_string(),
            &req.to_path,
            &content,
            &content_type,
        )
        .await?;

    Ok(StatusCode::OK)
}
