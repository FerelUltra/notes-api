use axum::{
    extract::State,
    http::StatusCode,
    Json,
};

use crate::{
    dto::{RegisterUserDto, LoginUserDto},
    errors::AppError,
    models::users::User,
    state::AppState,
};

pub async fn register(
    State(state): State<AppState>,
    Json(dto): Json<RegisterUserDto>
) -> Result<(StatusCode, Json<User>), AppError>{
    let user = state.user_service.register_user(dto).await?;

    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn login(
    State(state): State<AppState>,
    Json(dto): Json<LoginUserDto>,
) -> Result<Json<User>, AppError> {
    let user = state.user_service.login_user(dto).await?;

    Ok(Json(user))
}