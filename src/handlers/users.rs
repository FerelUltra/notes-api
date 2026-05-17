use axum::{
    extract::{Path, State},
    Json,
};
use redis::AsyncCommands;

use crate::{
    auth::extractor::AuthUser, dto::users::UpdateUserDto, errors::AppError, models::users::User,
    state::AppState,
};

const USERS_CACHE_KEY: &str = "users:all";
const USERS_CACHE_TTL_SECONDS: u64 = 60;

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

pub async fn get_users(
    AuthUser { user_id: _ }: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<User>>, AppError> {
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
    AuthUser { user_id: _ }: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<User>, AppError> {
    let user = state.user_service.get_user_by_id(id).await?;

    Ok(Json(user))
}

pub async fn update_user(
    AuthUser { user_id }: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(dto): Json<UpdateUserDto>,
) -> Result<Json<User>, AppError> {
    if user_id != id {
        return Err(AppError::Forbidden(
            "You can only update your own user".to_string(),
        ));
    }

    dto.validate()?;

    let user = state.user_service.update_user(id, dto).await?;

    invalidate_users_cache(&state).await;

    Ok(Json(user))
}

pub async fn delete_user(
    AuthUser { user_id }: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<String>, AppError> {
    if user_id != id {
        return Err(AppError::Forbidden(
            "You can only delete your own user".to_string(),
        ));
    }

    let deleted = state.user_service.delete_user(id).await?;

    if deleted {
        invalidate_users_cache(&state).await;

        Ok(Json(format!("User with id {} deleted", id)))
    } else {
        Err(AppError::NotFound(format!("User with id {} not found", id)))
    }
}
