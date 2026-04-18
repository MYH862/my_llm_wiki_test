use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::middleware::auth::Claims;

pub async fn require_permission(
    pool: axum::extract::State<PgPool>,
    claims: axum::extract::Extension<Claims>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user_id = claims.0.sub;
    
    let has_permission = check_user_permission(&pool, user_id, "admin")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !has_permission {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}

pub async fn check_user_permission(
    pool: &PgPool,
    user_id: Uuid,
    permission_name: &str,
) -> Result<bool, sqlx::Error> {
    let is_super_admin: Option<bool> = sqlx::query_scalar(
        "SELECT is_super_admin FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    if is_super_admin == Some(true) {
        return Ok(true);
    }

    let count: Option<i64> = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM user_roles ur
        JOIN role_permissions rp ON ur.role_id = rp.role_id
        JOIN permissions p ON rp.permission_id = p.id
        WHERE ur.user_id = $1 AND p.name = $2
        "#,
    )
    .bind(user_id)
    .bind(permission_name)
    .fetch_one(pool)
    .await?;

    Ok(count.unwrap_or(0) > 0)
}

pub async fn check_project_permission(
    pool: &PgPool,
    user_id: Uuid,
    project_id: Uuid,
    permission_name: &str,
) -> Result<bool, sqlx::Error> {
    let is_super_admin: Option<bool> = sqlx::query_scalar(
        "SELECT is_super_admin FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    if is_super_admin == Some(true) {
        return Ok(true);
    }

    let count: Option<i64> = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM project_members pm
        JOIN user_roles ur ON pm.user_id = ur.user_id
        JOIN role_permissions rp ON ur.role_id = rp.role_id
        JOIN permissions p ON rp.permission_id = p.id
        WHERE pm.user_id = $1 AND pm.project_id = $2 AND p.name = $3
        "#,
    )
    .bind(user_id)
    .bind(project_id)
    .bind(permission_name)
    .fetch_one(pool)
    .await?;

    Ok(count.unwrap_or(0) > 0)
}

pub async fn is_super_admin(pool: &PgPool, user_id: Uuid) -> Result<bool, sqlx::Error> {
    let result: Option<bool> = sqlx::query_scalar(
        "SELECT is_super_admin FROM users WHERE id = $1 AND is_active = true"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(result.unwrap_or(false))
}
