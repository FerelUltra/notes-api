use axum::{
    Json, extract::{ConnectInfo, Path, State}, http::StatusCode
};
use redis::AsyncCommands;
use std::net::SocketAddr;

use crate::{
    auth::extractor::AuthUser, dto::{RegisterUserDto, users::{CreateUserDto, UpdateUserDto}}, errors::AppError, models::users::User, state::AppState
};

const USERS_CACHE_KEY: &str = "users:all";
const USERS_CACHE_TTL_SECONDS: u64 = 60;

const CREATE_USER_RATE_LIMIT_MAX_REQUESTS: u64 = 5;
const CREATE_USER_RATE_LIMIT_WINDOW_SECONDS: i64 = 60;

async fn invalidate_users_cache(state: &AppState) {
    let Ok(mut conn) = state.redis.get_multiplexed_async_connection().await else {
        tracing::warn!("failed to connect to Redis for cache invalidation");
        return;
    };

    let result: redis::RedisResult<()> = conn.del(USERS_CACHE_KEY).await;

    if let Err(error) = result {
        tracing::warn!("failed to invalidate users cache: {}", error);
    }
}

async fn check_create_user_rate_limit(
    state: &AppState,
    client_ip: &str,
) -> Result<(), AppError> {
    let Ok(mut conn) = state.redis.get_multiplexed_async_connection().await else {
        tracing::warn!("failed to connect to Redis for rate limiting");
        return Ok(());
    };

    let key = format!("rate_limit:create_user:{}", client_ip);

    let requests_count_result: redis::RedisResult<u64> = conn.incr(&key, 1).await;

    let requests_count = match requests_count_result {
        Ok(count) => count,
        Err(error) => {
            tracing::warn!("failed to increment rate limit key: {}", error);
            return Ok(());
        }
    };

    if requests_count == 1 {
        let expire_result: redis::RedisResult<bool> = conn
            .expire(&key, CREATE_USER_RATE_LIMIT_WINDOW_SECONDS)
            .await;

        if let Err(error) = expire_result {
            tracing::warn!("failed to set rate limit TTL: {}", error);
        }
    }

    if requests_count > CREATE_USER_RATE_LIMIT_MAX_REQUESTS {
        tracing::warn!(
            "rate limit exceeded for POST /users from IP {}",
            client_ip
        );

        return Err(AppError::TooManyRequests(
            "Too many requests. Try again later.".to_string(),
        ));
    }

    Ok(())
}

pub async fn register(
    State(state): State<AppState>,
    Json(dto): Json<RegisterUserDto>,
) -> Result<(StatusCode, Json<User>), AppError> {
    let user = state.user_service.register_user(dto).await?;

    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn get_users(AuthUser {user_id: _}: AuthUser, State(state): State<AppState>,  ) -> Result<Json<Vec<User>>, AppError> {
    let redis_connection = state.redis.get_multiplexed_async_connection().await;

    if let Ok(mut conn) = redis_connection {
        let cached: redis::RedisResult<Option<String>> = conn.get(USERS_CACHE_KEY).await;

        if let Ok(Some(cached_users_json)) = cached {
            let parsed_users = serde_json::from_str::<Vec<User>>(&cached_users_json);

            if let Ok(users) = parsed_users {
                tracing::info!("users loaded from Redis cache");
                return Ok(Json(users));
            }

            tracing::warn!("failed to parse users from Redis cache");
        }
    } else {
        tracing::warn!("failed to connect to Redis, using PostgreSQL");
    }

    let users = state.user_service.get_users().await?;

    let users_json = serde_json::to_string(&users);

    if let Ok(users_json) = users_json {
        if let Ok(mut conn) = state.redis.get_multiplexed_async_connection().await {
            let result: redis::RedisResult<()> = conn
                .set_ex(USERS_CACHE_KEY, users_json, USERS_CACHE_TTL_SECONDS)
                .await;

            if let Err(error) = result {
                tracing::warn!("failed to save users to Redis cache: {}", error);
            } else {
                tracing::info!("users saved to Redis cache");
            }
        }
    }

    Ok(Json(users))
}

pub async fn get_user_by_id(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<User>, AppError> {
    let user = state.user_service.get_user_by_id(id).await?;

    Ok(Json(user))
}

pub async fn create_user(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(dto): Json<CreateUserDto>,
) -> Result<Json<User>, AppError> {
    let client_ip = addr.ip().to_string();

    check_create_user_rate_limit(&state, &client_ip).await?;

    dto.validate()?;

    let user = state.user_service.create_user(dto).await?;

    invalidate_users_cache(&state).await;

    Ok(Json(user))
}

pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(dto): Json<UpdateUserDto>,
) -> Result<Json<User>, AppError> {
    dto.validate()?;

    let user = state.user_service.update_user(id, dto).await?;

    invalidate_users_cache(&state).await;

    Ok(Json(user))
}

pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<String>, AppError> {
    let deleted = state.user_service.delete_user(id).await?;

    if deleted {
        invalidate_users_cache(&state).await;

        Ok(Json(format!("User with id {} deleted", id)))
    } else {
        Err(AppError::NotFound(format!("User with id {} not found", id)))
    }
}