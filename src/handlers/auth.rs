use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    Json,
};
use redis::AsyncCommands;
use std::net::SocketAddr;

use crate::{
    auth::extractor::AuthUser, db, dto::{
        AuthResponseDto, LoginUserDto, RegisterUserDto, auth::{LogoutDto, RefreshTokenDto}
    }, errors::AppError, jwt::{
        generate_access_token, generate_refresh_token, hash_refresh_token, refresh_token_expires_at,
    }, state::AppState
};

const REGISTER_RATE_LIMIT_MAX_REQUESTS: u64 = 5;
const REGISTER_RATE_LIMIT_WINDOW_SECONDS: i64 = 60;

async fn check_register_rate_limit(state: &AppState, client_ip: &str) -> Result<(), AppError> {
    let Ok(mut conn) = state.redis.get_multiplexed_async_connection().await else {
        tracing::warn!("failed to connect to Redis for register rate limiting");
        return Ok(());
    };

    let key = format!("rate_limit:register:{}", client_ip);

    let requests_count_result: redis::RedisResult<u64> = conn.incr(&key, 1).await;

    let requests_count = match requests_count_result {
        Ok(count) => count,
        Err(error) => {
            tracing::warn!("failed to increment register rate limit key: {}", error);
            return Ok(());
        }
    };

    if requests_count == 1 {
        let expire_result: redis::RedisResult<bool> =
            conn.expire(&key, REGISTER_RATE_LIMIT_WINDOW_SECONDS).await;

        if let Err(error) = expire_result {
            tracing::warn!("failed to set register rate limit TTL: {}", error);
        }
    }

    if requests_count > REGISTER_RATE_LIMIT_MAX_REQUESTS {
        tracing::warn!("register rate limit exceeded for IP {}", client_ip);

        return Err(AppError::TooManyRequests(
            "Too many requests. Try again later.".to_string(),
        ));
    }

    Ok(())
}

pub async fn register(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(dto): Json<RegisterUserDto>,
) -> Result<(StatusCode, Json<AuthResponseDto>), AppError> {
    let client_ip = addr.ip().to_string();

    check_register_rate_limit(&state, &client_ip).await?;

    dto.validate()?;

    let user = state.user_service.register_user(dto).await?;
    let access_token = generate_access_token(user.id, &state.jwt_secret, user.token_version)?;

    let refresh_token = generate_refresh_token();
    let refresh_token_hash = hash_refresh_token(&refresh_token);

    db::refresh_tokens::create_refresh_token(
        &state.db,
        user.id,
        &refresh_token_hash,
        refresh_token_expires_at(),
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(AuthResponseDto {
            access_token,
            token: "Bearer".to_string(),
            refresh_token,
        }),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    Json(dto): Json<LoginUserDto>,
) -> Result<Json<AuthResponseDto>, AppError> {
    dto.validate()?;
    let user = state.user_service.login_user(dto).await?;

    let access_token = generate_access_token(user.id, &state.jwt_secret, user.token_version)?;

    let refresh_token = generate_refresh_token();
    let refresh_token_hash = hash_refresh_token(&refresh_token);

    db::refresh_tokens::create_refresh_token(
        &state.db,
        user.id,
        &refresh_token_hash,
        refresh_token_expires_at(),
    )
    .await?;

    Ok(Json(AuthResponseDto {
        access_token,
        token: "Bearer".to_string(),
        refresh_token,
    }))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(dto): Json<RefreshTokenDto>,
) -> Result<Json<AuthResponseDto>, AppError> {
    let old_token_hash = hash_refresh_token(&dto.refresh_token);

    let stored_token = db::refresh_tokens::find_valid_refresh_token(&state.db, &old_token_hash)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid refresh token".to_string()))?;

    db::refresh_tokens::revoke_refresh_token(&state.db, &old_token_hash).await?;

    let token_version = db::users::get_user_token_version(&state.db, stored_token.user_id).await?.ok_or_else(|| AppError::Unauthorized("Invalid refresh token".to_string()))?;
    let access_token = generate_access_token(stored_token.user_id, &state.jwt_secret, token_version)?;

    let refresh_token = generate_refresh_token();
    let refresh_token_hash = hash_refresh_token(&refresh_token);

    db::refresh_tokens::create_refresh_token(
        &state.db,
        stored_token.user_id,
        &refresh_token_hash,
        refresh_token_expires_at(),
    )
    .await?;

    Ok(Json(AuthResponseDto {
        access_token,
        refresh_token,
        token: "Bearer".to_string(),
    }))
}

pub async fn logout(
    State(state): State<AppState>,
    Json(dto): Json<LogoutDto>,
) -> Result<StatusCode, AppError> {
    let token_hash = hash_refresh_token(&dto.refresh_token);

    db::refresh_tokens::revoke_refresh_token(&state.db, &token_hash).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn logout_all(
    AuthUser { user_id}: AuthUser,
    State(state): State<AppState>,
) -> Result<StatusCode, AppError> {
    db::refresh_tokens::revoke_all_refresh_tokens_for_user(&state.db, user_id).await?;

    let updated = db::users::increment_token_version(&state.db, user_id).await?;

    if !updated{
        return Err(AppError::Unauthorized("Invalid token".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}