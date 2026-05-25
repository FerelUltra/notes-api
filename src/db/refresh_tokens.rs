use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::errors::AppError;

#[derive(sqlx::FromRow)]
pub struct RefreshTokenRecord {
    pub id: i32,
    pub user_id: i32,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

pub async fn create_refresh_token(
    pool: &PgPool,
    user_id: i32,
    token_hash: &str,
    expires_at: DateTime<Utc>,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(user_id)
    .bind(token_hash)
    .bind(expires_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn find_valid_refresh_token(
    pool: &PgPool,
    token_hash: &str,
) -> Result<Option<RefreshTokenRecord>, AppError> {
    let token = sqlx::query_as::<_, RefreshTokenRecord>(
        r#"
        SELECT id, user_id, token_hash, expires_at, revoked_at
        FROM refresh_tokens
        WHERE token_hash = $1
            AND revoked_at IS NULL
            AND expires_at > NOW()
        "#,
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await?;

    Ok(token)
}

pub async fn revoke_refresh_token(pool: &PgPool, token_hash: &str) -> Result<bool, AppError> {
    let result = sqlx::query(
        r#"
            UPDATE refresh_tokens
            SET revoked_at = NOW()
            WHERE token_hash = $1
               AND revoked_at IS NULL
        "#,
    )
    .bind(token_hash)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn revoke_all_refresh_tokens_for_user(
    pool: &PgPool,
    user_id: i32,
) -> Result<u64, AppError> {
    let result = sqlx::query(
        r#"
        UPDATE refresh_tokens
        SET revoked_at = NOW()
        WHERE user_id = $1
            AND revoked_at IS NULL
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}