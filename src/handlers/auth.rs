use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    Json,
};
use redis::AsyncCommands;
use std::net::SocketAddr;

use crate::{
    dto::{AuthResponseDto, LoginUserDto, RegisterUserDto},
    errors::AppError,
    jwt::generate_access_token,
    state::AppState,
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
    let access_token = generate_access_token(user.id, &state.jwt_secret)?;

    Ok((
        StatusCode::CREATED,
        Json(AuthResponseDto {
            access_token,
            token: "Bearer".to_string(),
        }),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    Json(dto): Json<LoginUserDto>,
) -> Result<Json<AuthResponseDto>, AppError> {
    dto.validate()?;
    let user = state.user_service.login_user(dto).await?;

    let access_token = generate_access_token(user.id, &state.jwt_secret)?;

    Ok(Json(AuthResponseDto {
        access_token,
        token: "Bearer".to_string(),
    }))
}
