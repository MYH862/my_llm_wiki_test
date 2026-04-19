use sqlx::PgPool;
use uuid::Uuid;

use crate::middleware::auth::{generate_access_token, generate_refresh_token, Claims};
use crate::middleware::error::AppError;
use crate::models::user::{CreateUserRequest, User};

pub async fn register(
    pool: &PgPool,
    req: CreateUserRequest,
    jwt_secret: &str,
    jwt_expiration_hours: u64,
    jwt_refresh_expiration_days: u64,
) -> Result<(User, String, String), AppError> {
    let password_hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)
        .map_err(|_| AppError::Internal)?;

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (username, email, password_hash, display_name)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
    )
    .bind(&req.username)
    .bind(&req.email)
    .bind(&password_hash)
    .bind(&req.display_name)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.is_unique_violation() {
                return AppError::BadRequest("Username or email already exists".to_string());
            }
        }
        AppError::Internal
    })?;

    let access_token = generate_access_token(user.id, &user.username, jwt_secret, jwt_expiration_hours)
        .map_err(|_| AppError::Internal)?;

    let (refresh_token, jti) = generate_refresh_token(user.id, jwt_secret, jwt_refresh_expiration_days)
        .map_err(|_| AppError::Internal)?;

    let expires_at = chrono::Utc::now() + chrono::Duration::days(jwt_refresh_expiration_days as i64);

    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (user_id, token, expires_at, jti)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(user.id)
    .bind(&refresh_token)
    .bind(expires_at)
    .bind(jti)
    .execute(pool)
    .await
    .map_err(|_| AppError::Internal)?;

    Ok((user, access_token, refresh_token))
}

pub async fn login(
    pool: &PgPool,
    username: &str,
    password: &str,
    jwt_secret: &str,
    jwt_expiration_hours: u64,
    jwt_refresh_expiration_days: u64,
) -> Result<(User, String, String), AppError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1 AND is_active = true")
        .bind(username)
        .fetch_optional(pool)
        .await
        .map_err(|_| AppError::Internal)?;

    let user = user.ok_or_else(|| AppError::Auth("Invalid credentials".to_string()))?;

    let valid = bcrypt::verify(password, &user.password_hash)
        .map_err(|_| AppError::Internal)?;

    if !valid {
        return Err(AppError::Auth("Invalid credentials".to_string()));
    }

    let access_token = generate_access_token(user.id, &user.username, jwt_secret, jwt_expiration_hours)
        .map_err(|_| AppError::Internal)?;

    let (refresh_token, jti) = generate_refresh_token(user.id, jwt_secret, jwt_refresh_expiration_days)
        .map_err(|_| AppError::Internal)?;

    let expires_at = chrono::Utc::now() + chrono::Duration::days(jwt_refresh_expiration_days as i64);

    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (user_id, token, expires_at, jti)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(user.id)
    .bind(&refresh_token)
    .bind(expires_at)
    .bind(jti)
    .execute(pool)
    .await
    .map_err(|_| AppError::Internal)?;

    Ok((user, access_token, refresh_token))
}

pub async fn refresh_token(
    pool: &PgPool,
    token: &str,
    jwt_secret: &str,
    jwt_expiration_hours: u64,
) -> Result<(String, String), AppError> {
    let stored_token = sqlx::query!(
        r#"
        SELECT rt.*, u.username
        FROM refresh_tokens rt
        JOIN users u ON rt.user_id = u.id
        WHERE rt.token = $1 AND rt.is_revoked = false AND rt.expires_at > NOW()
        "#,
        token
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?;

    let stored_token = stored_token.ok_or_else(|| AppError::Auth("Invalid or expired refresh token".to_string()))?;

    sqlx::query!("UPDATE refresh_tokens SET is_revoked = true WHERE token = $1", token)
        .execute(pool)
        .await
        .map_err(|_| AppError::Internal)?;

    let user_id: Uuid = stored_token.user_id;
    let username: String = stored_token.username;

    let new_access_token = generate_access_token(user_id, &username, jwt_secret, jwt_expiration_hours)
        .map_err(|_| AppError::Internal)?;

    let (new_refresh_token, jti) = generate_refresh_token(user_id, jwt_secret, 7)
        .map_err(|_| AppError::Internal)?;

    let expires_at = chrono::Utc::now() + chrono::Duration::days(7);

    sqlx::query!(
        r#"
        INSERT INTO refresh_tokens (user_id, token, expires_at, jti)
        VALUES ($1, $2, $3, $4)
        "#,
        user_id,
        new_refresh_token,
        expires_at,
        jti
    )
    .execute(pool)
    .await
    .map_err(|_| AppError::Internal)?;

    Ok((new_access_token, new_refresh_token))
}

pub async fn logout(pool: &PgPool, token: &str) -> Result<(), AppError> {
    sqlx::query!("UPDATE refresh_tokens SET is_revoked = true WHERE token = $1", token)
        .execute(pool)
        .await
        .map_err(|_| AppError::Internal)?;

    Ok(())
}

pub async fn change_password(
    pool: &PgPool,
    user_id: Uuid,
    current_password: &str,
    new_password: &str,
) -> Result<(), AppError> {
    let user = sqlx::query_as::<_, crate::models::user::User>(
        "SELECT * FROM users WHERE id = $1 AND is_active = true"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::Internal)?;

    let user = user.ok_or_else(|| AppError::Auth("User not found".to_string()))?;

    let valid = bcrypt::verify(current_password, &user.password_hash)
        .map_err(|_| AppError::Internal)?;

    if !valid {
        return Err(AppError::Auth("Current password is incorrect".to_string()));
    }

    let new_password_hash = bcrypt::hash(new_password, bcrypt::DEFAULT_COST)
        .map_err(|_| AppError::Internal)?;

    sqlx::query!(
        "UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2",
        new_password_hash,
        user_id
    )
    .execute(pool)
    .await
    .map_err(|_| AppError::Internal)?;

    Ok(())
}
