use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json, Router,
    routing::{get, post, delete},
};
use serde::Deserialize;
use uuid::Uuid;
use std::io::Cursor;

use crate::config::AppState;
use crate::middleware::auth::Claims;
use crate::middleware::error::AppError;
use crate::middleware::permission::check_project_permission;
use crate::models::file::{FileInfo, DirectoryEntry, FileContent, DocumentPreprocessResponse};

fn extract_pdf_text(content: &[u8]) -> Result<String, AppError> {
    let doc = lopdf::Document::load_from(Cursor::new(content))
        .map_err(|e| AppError::BadRequest(format!("Failed to parse PDF: {}", e)))?;

    let mut text = String::new();
    for page in doc.get_pages() {
        let page_number = page.0;
        if let Ok(page_text) = doc.extract_text(&[page_number]) {
            text.push_str(&page_text);
            text.push_str("\n\n");
        }
    }

    Ok(text)
}

fn extract_excel_text(content: &[u8]) -> Result<String, AppError> {
    let mut workbook = calamine::Xlsx::<Cursor<&[u8]>>::new(Cursor::new(content))
        .map_err(|e| AppError::BadRequest(format!("Failed to parse Excel: {}", e)))?;

    let mut text = String::new();
    for sheet_name in workbook.sheet_names() {
        if let Ok(range) = workbook.worksheet_range(&sheet_name) {
            text.push_str(&format!("## Sheet: {}\n\n", sheet_name));
            for row in range.rows() {
                let row_text: Vec<String> = row
                    .iter()
                    .map(|cell| cell.to_string())
                    .collect();
                text.push_str(&row_text.join("\t"));
                text.push_str("\n");
            }
            text.push_str("\n");
        }
    }

    Ok(text)
}

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

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/list", get(list_files))
        .route("/read", get(read_file))
        .route("/write", post(write_file))
        .route("/delete", delete(delete_file))
        .route("/copy", post(copy_file))
        .route("/preprocess", get(preprocess_document))
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

pub async fn preprocess_document(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Path(project_id): Path<Uuid>,
    Query(query): Query<ReadQuery>,
) -> Result<Json<DocumentPreprocessResponse>, AppError> {
    check_project_permission(
        &state.db,
        claims.0.sub,
        project_id,
        "files:read",
    )
    .await
    .map_err(|_| AppError::PermissionDenied)?;

    let content = state.minio.download_file(&project_id.to_string(), &query.path).await?;
    
    let content_type = if query.path.ends_with(".pdf") {
        "application/pdf"
    } else if query.path.ends_with(".docx") {
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
    } else if query.path.ends_with(".pptx") {
        "application/vnd.openxmlformats-officedocument.presentationml.presentation"
    } else if query.path.ends_with(".xlsx") || query.path.ends_with(".xls") {
        "application/vnd.ms-excel"
    } else {
        return Err(AppError::BadRequest("Unsupported file format".to_string()));
    };

    let markdown_content = match content_type {
        "application/pdf" => {
            extract_pdf_text(&content)?
        }
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" | 
        "application/vnd.ms-excel" => {
            extract_excel_text(&content)?
        }
        _ => String::from_utf8(content).map_err(|e| AppError::BadRequest(e.to_string()))?,
    };

    Ok(Json(DocumentPreprocessResponse {
        original_path: query.path,
        content_type: content_type.to_string(),
        markdown_content,
        page_count: None,
    }))
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
