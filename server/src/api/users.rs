use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json, Router,
    routing::{get, put, delete},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::AppState;
use crate::middleware::auth::Claims;
use crate::middleware::error::AppError;
use crate::middleware::permission::{check_user_permission, is_super_admin};
use crate::models::user::{User, UpdateUserRequest};

#[derive(Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub is_super_admin: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            display_name: user.display_name,
            avatar_url: user.avatar_url,
            is_active: user.is_active,
            is_super_admin: user.is_super_admin,
            created_at: user.created_at,
        }
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users))
        .route("/:id", get(get_user))
        .route("/:id", put(update_user))
        .route("/:id", delete(delete_user))
}

pub async fn list_users(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
) -> Result<Json<Vec<UserResponse>>, AppError> {
    if !check_user_permission(&state.db, claims.0.sub, "users:read").await? {
        return Err(AppError::PermissionDenied);
    }

    let users = sqlx::query_as::<_, User>(
        "SELECT * FROM users ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| AppError::Internal)?;

    Ok(Json(users.into_iter().map(UserResponse::from).collect()))
}

pub async fn get_user(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserResponse>, AppError> {
    if !check_user_permission(&state.db, claims.0.sub, "users:read").await? {
        return Err(AppError::PermissionDenied);
    }

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| AppError::Internal)?;

    match user {
        Some(u) => Ok(Json(UserResponse::from(u))),
        None => Err(AppError::NotFound("User not found".to_string())),
    }
}

pub async fn update_user(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    let is_admin = check_user_permission(&state.db, claims.0.sub, "users:update").await?;
    let is_self = claims.0.sub == id;

    if !is_admin && !is_self {
        return Err(AppError::PermissionDenied);
    }

    let user = sqlx::query_as::<_, User>(
        r#"
        UPDATE users 
        SET display_name = COALESCE($1, display_name),
            avatar_url = COALESCE($2, avatar_url),
            email = COALESCE($3, email),
            updated_at = NOW()
        WHERE id = $4
        RETURNING *
        "#
    )
    .bind(&req.display_name)
    .bind(&req.avatar_url)
    .bind(&req.email)
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| AppError::Internal)?;

    match user {
        Some(u) => Ok(Json(UserResponse::from(u))),
        None => Err(AppError::NotFound("User not found".to_string())),
    }
}

pub async fn delete_user(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    if !check_user_permission(&state.db, claims.0.sub, "users:delete").await? {
        return Err(AppError::PermissionDenied);
    }

    if is_super_admin(&state.db, id).await? {
        return Err(AppError::BadRequest("Cannot delete super admin user".to_string()));
    }

    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|_| AppError::Internal)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}
