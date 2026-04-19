use axum::{
    extract::Request,
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::config::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub username: String,
    pub exp: usize,
    pub iat: usize,
    pub token_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: Uuid,
    pub exp: usize,
    pub iat: usize,
    pub jti: Uuid,
}

pub async fn auth_middleware(
    state: axum::extract::State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let auth_str = auth_header
        .to_str()
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    if !auth_str.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = &auth_str[7..];

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.config.jwt.secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    if token_data.claims.token_type != "access" {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let mut request = request;
    request.extensions_mut().insert(token_data.claims);

    Ok(next.run(request).await)
}

pub fn generate_access_token(
    user_id: Uuid,
    username: &str,
    secret: &str,
    expiration_hours: u64,
) -> Result<String, StatusCode> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        exp: (now.as_secs() + expiration_hours * 3600) as usize,
        iat: now.as_secs() as usize,
        token_type: "access".to_string(),
    };

    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub fn generate_refresh_token(
    user_id: Uuid,
    secret: &str,
    expiration_days: u64,
) -> Result<(String, Uuid), StatusCode> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let jti = Uuid::new_v4();

    let claims = RefreshClaims {
        sub: user_id,
        exp: (now.as_secs() + expiration_days * 86400) as usize,
        iat: now.as_secs() as usize,
        jti,
    };

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((token, jti))
}
