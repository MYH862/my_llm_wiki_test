use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json, Router,
    routing::{get, post, put, delete},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::AppState;
use crate::middleware::auth::Claims;
use crate::middleware::error::AppError;
use crate::middleware::permission::{check_user_permission, check_project_permission, is_super_admin};
use crate::models::project::{Project, CreateProjectRequest, UpdateProjectRequest, AddProjectMemberRequest, ProjectMember};

#[derive(Serialize)]
pub struct ProjectResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub storage_path: String,
    pub template: Option<String>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<Project> for ProjectResponse {
    fn from(project: Project) -> Self {
        Self {
            id: project.id,
            name: project.name,
            description: project.description,
            owner_id: project.owner_id,
            storage_path: project.storage_path,
            template: project.template,
            is_active: project.is_active,
            created_at: project.created_at,
        }
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_projects))
        .route("/", post(create_project))
        .route("/:id", get(get_project))
        .route("/:id", put(update_project))
        .route("/:id", delete(delete_project))
        .route("/:id/members", get(list_project_members))
        .route("/:id/members", post(add_project_member))
        .route("/:id/members/:user_id", delete(remove_project_member))
}

pub async fn list_projects(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
) -> Result<Json<Vec<ProjectResponse>>, AppError> {
    let user_id = claims.0.sub;
    
    let is_admin = is_super_admin(&state.db, user_id).await?;
    
    let projects = if is_admin {
        sqlx::query_as::<_, Project>(
            "SELECT * FROM projects WHERE is_active = true ORDER BY created_at DESC"
        )
        .fetch_all(&state.db)
        .await
        .map_err(|_| AppError::Internal)?
    } else {
        sqlx::query_as::<_, Project>(
            r#"
            SELECT DISTINCT p.* 
            FROM projects p
            LEFT JOIN project_members pm ON p.id = pm.project_id
            WHERE p.is_active = true AND (p.owner_id = $1 OR pm.user_id = $1)
            ORDER BY p.created_at DESC
            "#
        )
        .bind(user_id)
        .fetch_all(&state.db)
        .await
        .map_err(|_| AppError::Internal)?
    };

    Ok(Json(projects.into_iter().map(ProjectResponse::from).collect()))
}

pub async fn create_project(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<ProjectResponse>), AppError> {
    if !check_user_permission(&state.db, claims.0.sub, "projects:create").await? {
        return Err(AppError::PermissionDenied);
    }

    let storage_path = format!("/projects/{}", Uuid::new_v4());

    let project = sqlx::query_as::<_, Project>(
        r#"
        INSERT INTO projects (name, description, owner_id, storage_path, template)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#
    )
    .bind(&req.name)
    .bind(&req.description)
    .bind(claims.0.sub)
    .bind(&storage_path)
    .bind(&req.template)
    .fetch_one(&state.db)
    .await
    .map_err(|_| AppError::Internal)?;

    state.minio.ensure_bucket(&project.id.to_string()).await?;

    Ok((StatusCode::CREATED, Json(ProjectResponse::from(project))))
}

pub async fn get_project(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProjectResponse>, AppError> {
    if !check_project_permission(&state.db, claims.0.sub, id, "projects:read").await? {
        return Err(AppError::PermissionDenied);
    }

    let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| AppError::Internal)?;

    match project {
        Some(p) => Ok(Json(ProjectResponse::from(p))),
        None => Err(AppError::NotFound("Project not found".to_string())),
    }
}

pub async fn update_project(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateProjectRequest>,
) -> Result<Json<ProjectResponse>, AppError> {
    if !check_project_permission(&state.db, claims.0.sub, id, "projects:update").await? {
        return Err(AppError::PermissionDenied);
    }

    let project = sqlx::query_as::<_, Project>(
        r#"
        UPDATE projects 
        SET name = COALESCE($1, name),
            description = COALESCE($2, description),
            updated_at = NOW()
        WHERE id = $3
        RETURNING *
        "#
    )
    .bind(&req.name)
    .bind(&req.description)
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| AppError::Internal)?;

    match project {
        Some(p) => Ok(Json(ProjectResponse::from(p))),
        None => Err(AppError::NotFound("Project not found".to_string())),
    }
}

pub async fn delete_project(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    if !check_project_permission(&state.db, claims.0.sub, id, "projects:delete").await? {
        return Err(AppError::PermissionDenied);
    }

    sqlx::query("UPDATE projects SET is_active = false, updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|_| AppError::Internal)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_project_members(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<ProjectMember>>, AppError> {
    if !check_project_permission(&state.db, claims.0.sub, project_id, "projects:manage_members").await? {
        return Err(AppError::PermissionDenied);
    }

    let members = sqlx::query_as::<_, ProjectMember>(
        r#"
        SELECT 
            pm.project_id,
            pm.user_id,
            pm.role_id,
            u.username,
            u.display_name,
            r.name as role_name
        FROM project_members pm
        JOIN users u ON pm.user_id = u.id
        JOIN roles r ON pm.role_id = r.id
        WHERE pm.project_id = $1
        "#
    )
    .bind(project_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| AppError::Internal)?;

    Ok(Json(members))
}

pub async fn add_project_member(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<AddProjectMemberRequest>,
) -> Result<StatusCode, AppError> {
    if !check_project_permission(&state.db, claims.0.sub, project_id, "projects:manage_members").await? {
        return Err(AppError::PermissionDenied);
    }

    sqlx::query(
        r#"
        INSERT INTO project_members (project_id, user_id, role_id)
        VALUES ($1, $2, $3)
        ON CONFLICT (project_id, user_id) DO UPDATE SET role_id = $3
        "#
    )
    .bind(project_id)
    .bind(req.user_id)
    .bind(req.role_id)
    .execute(&state.db)
    .await
    .map_err(|_| AppError::Internal)?;

    Ok(StatusCode::OK)
}

pub async fn remove_project_member(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Path((project_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    if !check_project_permission(&state.db, claims.0.sub, project_id, "projects:manage_members").await? {
        return Err(AppError::PermissionDenied);
    }

    let result = sqlx::query(
        "DELETE FROM project_members WHERE project_id = $1 AND user_id = $2"
    )
    .bind(project_id)
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(|_| AppError::Internal)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Member not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}
