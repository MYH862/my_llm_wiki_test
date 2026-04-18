use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};
use serde::Deserialize;
use validator::Validate;

use crate::config::AppState;
use crate::middleware::auth::Claims;
use crate::middleware::error::AppError;
use crate::models::user::{CreateUserRequest, ChangePasswordRequest};
use crate::services::auth;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
        .route("/password", put(change_password))
}

#[derive(Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(length(min = 6))]
    pub password: String,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (user, access_token, refresh_token) = auth::register(
        &state.db,
        req,
        &state.config.jwt.secret,
        state.config.jwt.expiration_hours,
        state.config.jwt.refresh_expiration_days,
    )
    .await?;

    Ok(Json(serde_json::json!({
        "user": {
            "id": user.id,
            "username": user.username,
            "email": user.email,
            "display_name": user.display_name,
        },
        "access_token": access_token,
        "refresh_token": refresh_token,
    })))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (user, access_token, refresh_token) = auth::login(
        &state.db,
        &req.username,
        &req.password,
        &state.config.jwt.secret,
        state.config.jwt.expiration_hours,
        state.config.jwt.refresh_expiration_days,
    )
    .await?;

    Ok(Json(serde_json::json!({
        "user": {
            "id": user.id,
            "username": user.username,
            "email": user.email,
            "display_name": user.display_name,
        },
        "access_token": access_token,
        "refresh_token": refresh_token,
    })))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (access_token, refresh_token) = auth::refresh_token(
        &state.db,
        &req.refresh_token,
        &state.config.jwt.secret,
        state.config.jwt.expiration_hours,
    )
    .await?;

    Ok(Json(serde_json::json!({
        "access_token": access_token,
        "refresh_token": refresh_token,
    })))
}

pub async fn logout(
    State(state): State<AppState>,
    Json(req): Json<LogoutRequest>,
) -> Result<StatusCode, AppError> {
    auth::logout(&state.db, &req.refresh_token).await?;
    Ok(StatusCode::OK)
}

pub async fn change_password(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<StatusCode, AppError> {
    auth::change_password(
        &state.db,
        claims.0.sub,
        &req.current_password,
        &req.new_password,
    )
    .await?;

    Ok(StatusCode::OK)
}
