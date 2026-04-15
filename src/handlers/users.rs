use axum::{
	extract::{Path, State},
	Json,
};

use crate::{
	db,
	dto::{CreateUserDto, UpdateUserDto},
	models::User,
	state::AppState,
	errors::AppError
};

pub async fn get_users(
	State(state): State<AppState>,
) -> Result<Json<Vec<User>>, AppError> {
	let users = db::users::get_users(&state.db).await?;

	Ok(Json(users))
}

pub async fn get_user_by_id(
	State(state): State<AppState>,
	Path(id): Path<i32>,
) -> Result<Json<User>, AppError> {
	let user = db::users::get_user_by_id(&state.db, id).await?;

	match user{
		Some(user) => Ok(Json(user)),
		None =>	Err(AppError::NotFound(format!("User with id {} not found", id)))
	}
}

pub async fn create_user(
	State(state): State<AppState>,
	Json(dto): Json<CreateUserDto>,

) -> Result<Json<User>, AppError>{

	let user = db::users::create_user(&state.db, dto).await?;
	Ok(Json(user))
}

pub async fn update_user(
	State(state): State<AppState>,
	Path(id): Path<i32>,
	Json(dto): Json<UpdateUserDto>,
) -> Result<Json<User>, AppError>{
	let user = db::users::update_user(&state.db, id, dto).await?;

	match user {
		Some(user) => Ok(Json(user)),
		None => Err(AppError::NotFound(format!("User with id {} not found", id)))
	}
}

pub async fn delete_user(
	State(state): State<AppState>,
	Path(id): Path<i32>,
) -> Result<Json<String>, AppError>{
	let deleted = db::users::delete_user(&state.db, id).await?;

	if deleted {
		Ok(Json(format!("User with id {} deleted", id)))
	} else {
		Err(AppError::NotFound(format!("User with id {} not found", id)))
	}
}