use axum::{
    extract::{Path, State},
    Json,
};

use crate::{
    dto::users::{CreateUserDto, UpdateUserDto},
    errors::AppError,
    models::users::User,
    state::AppState,
};

pub async fn get_users(State(state): State<AppState>) -> Result<Json<Vec<User>>, AppError> {
    let users = state.user_service.get_users().await?;

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
    Json(dto): Json<CreateUserDto>,
) -> Result<Json<User>, AppError> {
    dto.validate()?;

    let user = state.user_service.create_user( dto).await?;
    Ok(Json(user))
}

pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(dto): Json<UpdateUserDto>,
) -> Result<Json<User>, AppError> {
    dto.validate()?;

    let user = state.user_service.update_user( id, dto).await?;

    Ok(Json(user))
}

pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<String>, AppError> {
    let deleted = state.user_service.delete_user( id).await?;

    if deleted {
        Ok(Json(format!("User with id {} deleted", id)))
    } else {
        Err(AppError::NotFound(format!("User with id {} not found", id)))
    }
}
