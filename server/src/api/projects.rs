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
use crate::models::project::{
    Project, CreateProjectRequest, UpdateProjectRequest, AddProjectMemberRequest, 
    ProjectMember, ProjectResponse, ProjectSettings, UpdateProjectSettingsRequest, 
    ProjectStats, OpenProjectResponse
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_projects))
        .route("/", post(create_project))
        .route("/:id", get(get_project))
        .route("/:id", put(update_project))
        .route("/:id", delete(delete_project))
        .route("/:id/open", get(open_project))
        .route("/:id/settings", get(get_project_settings))
        .route("/:id/settings", put(update_project_settings))
        .route("/:id/stats", get(get_project_stats))
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

pub async fn open_project(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<OpenProjectResponse>, AppError> {
    if !check_project_permission(&state.db, claims.0.sub, id, "projects:read").await? {
        return Err(AppError::PermissionDenied);
    }

    let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| AppError::Internal)?;

    let project = match project {
        Some(p) => p,
        None => return Err(AppError::NotFound("Project not found".to_string())),
    };

    let settings = sqlx::query_as::<_, (String, Option<String>)>(
        r#"
        SELECT key, value FROM project_settings WHERE project_id = $1
        "#
    )
    .bind(id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let settings_map: std::collections::HashMap<String, Option<String>> = settings
        .into_iter()
        .map(|(k, v)| (k, v))
        .collect();

    let project_settings = ProjectSettings {
        llm_provider: settings_map.get("llm_provider").cloned().flatten(),
        llm_api_key: settings_map.get("llm_api_key").cloned().flatten(),
        llm_model: settings_map.get("llm_model").cloned().flatten(),
        embedding_model: settings_map.get("embedding_model").cloned().flatten(),
        vector_db_url: settings_map.get("vector_db_url").cloned().flatten(),
        max_context_tokens: settings_map.get("max_context_tokens")
            .cloned()
            .flatten()
            .and_then(|v| v.parse().ok()),
        enable_auto_index: settings_map.get("enable_auto_index")
            .cloned()
            .flatten()
            .and_then(|v| v.parse().ok()),
    };

    let stats = get_project_stats_internal(&state, id).await;

    let user_role = sqlx::query_scalar::<_, String>(
        r#"
        SELECT r.name FROM roles r
        JOIN project_members pm ON r.id = pm.role_id
        WHERE pm.project_id = $1 AND pm.user_id = $2
        "#
    )
    .bind(id)
    .bind(claims.0.sub)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| AppError::Internal)?
    .unwrap_or_else(|| "viewer".to_string());

    Ok(Json(OpenProjectResponse {
        project: ProjectResponse::from(project),
        settings: project_settings,
        stats,
        user_role,
    }))
}

pub async fn get_project_settings(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProjectSettings>, AppError> {
    if !check_project_permission(&state.db, claims.0.sub, id, "projects:read").await? {
        return Err(AppError::PermissionDenied);
    }

    let settings = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT key, value FROM project_settings WHERE project_id = $1"
    )
    .bind(id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let settings_map: std::collections::HashMap<String, Option<String>> = settings
        .into_iter()
        .map(|(k, v)| (k, v))
        .collect();

    Ok(Json(ProjectSettings {
        llm_provider: settings_map.get("llm_provider").cloned().flatten(),
        llm_api_key: settings_map.get("llm_api_key").cloned().flatten(),
        llm_model: settings_map.get("llm_model").cloned().flatten(),
        embedding_model: settings_map.get("embedding_model").cloned().flatten(),
        vector_db_url: settings_map.get("vector_db_url").cloned().flatten(),
        max_context_tokens: settings_map.get("max_context_tokens")
            .cloned()
            .flatten()
            .and_then(|v| v.parse().ok()),
        enable_auto_index: settings_map.get("enable_auto_index")
            .cloned()
            .flatten()
            .and_then(|v| v.parse().ok()),
    }))
}

pub async fn update_project_settings(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateProjectSettingsRequest>,
) -> Result<Json<ProjectSettings>, AppError> {
    if !check_project_permission(&state.db, claims.0.sub, id, "projects:update").await? {
        return Err(AppError::PermissionDenied);
    }

    let settings_to_update = vec![
        ("llm_provider", req.llm_provider),
        ("llm_api_key", req.llm_api_key),
        ("llm_model", req.llm_model),
        ("embedding_model", req.embedding_model),
        ("vector_db_url", req.vector_db_url),
        ("max_context_tokens", req.max_context_tokens.map(|v| v.to_string())),
        ("enable_auto_index", req.enable_auto_index.map(|v| v.to_string())),
    ];

    for (key, value) in settings_to_update {
        if let Some(val) = value {
            sqlx::query(
                r#"
                INSERT INTO project_settings (project_id, key, value)
                VALUES ($1, $2, $3)
                ON CONFLICT (project_id, key) DO UPDATE SET value = $3, updated_at = NOW()
                "#
            )
            .bind(id)
            .bind(key)
            .bind(&val)
            .execute(&state.db)
            .await
            .map_err(|_| AppError::Internal)?;
        }
    }

    get_project_settings(State(state), claims, Path(id)).await
}

pub async fn get_project_stats(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProjectStats>, AppError> {
    if !check_project_permission(&state.db, claims.0.sub, id, "projects:read").await? {
        return Err(AppError::PermissionDenied);
    }

    Ok(Json(get_project_stats_internal(&state, id).await))
}

async fn get_project_stats_internal(state: &AppState, project_id: Uuid) -> ProjectStats {
    let total_files = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM files WHERE project_id = $1"
    )
    .bind(project_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
    .unwrap_or(0);

    let total_vectors = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM vectors WHERE project_id = $1"
    )
    .bind(project_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
    .unwrap_or(0);

    let total_graph_nodes = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM graph_nodes WHERE project_id = $1"
    )
    .bind(project_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
    .unwrap_or(0);

    let total_graph_edges = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM graph_edges WHERE project_id = $1"
    )
    .bind(project_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
    .unwrap_or(0);

    let last_indexed_at = sqlx::query_scalar::<_, chrono::DateTime<chrono::Utc>>(
        "SELECT MAX(updated_at) FROM files WHERE project_id = $1 AND indexed_at IS NOT NULL"
    )
    .bind(project_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    ProjectStats {
        total_files,
        total_vectors,
        total_graph_nodes,
        total_graph_edges,
        last_indexed_at,
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
