use axum::{extract::State, http::StatusCode, Json};

use crate::{
    dto::{AuthResponse, LoginUserDto, RegisterUserDto},
    errors::AppError,
    jwt::generate_access_token,
    models::users::User,
    state::AppState,
};

pub async fn register(
    State(state): State<AppState>,
    Json(dto): Json<RegisterUserDto>,
) -> Result<(StatusCode, Json<User>), AppError> {
    let user = state.user_service.register_user(dto).await?;

    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn login(
    State(state): State<AppState>,
    Json(dto): Json<LoginUserDto>,
) -> Result<Json<AuthResponse>, AppError> {
    let user = state.user_service.login_user(dto).await?;

    let access_token = generate_access_token(user.id, &state.jwt_secret)?;

    Ok(Json(AuthResponse {
        access_token,
        token_type: "Bearer".to_string(),
    }))
}
