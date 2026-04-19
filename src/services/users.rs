use sqlx::PgPool;

use crate::{
	db,
	dto::users::{CreateUserDto, UpdateUserDto,},
	errors::AppError,
	models::users::User,
};

#[derive(Clone)]
pub struct UserService {
	pool: PgPool
}

impl UserService{
	pub fn new(pool: PgPool) -> Self {
		Self {pool}
	}
	
	pub async fn create_user(&self, dto: CreateUserDto) -> Result<User, AppError> {
		let created_user = db::users::create_user(&self.pool, dto).await?;
		Ok(created_user)
	}

	pub async fn get_user_by_id(&self, user_id: i32) -> Result<User, AppError> {
		let user = db::users::get_user_by_id(&self.pool, user_id)
		.await?
		.ok_or(AppError::NotFound(format!("User with id {} not found", user_id)))?;
		Ok(user)
	}

	pub async fn get_users(&self) -> Result<Vec<User>, AppError> {
		let users = db::users::get_users(&self.pool).await?;
		Ok(users)
	}

	pub async fn update_user(&self, user_id: i32, dto: UpdateUserDto) -> Result<User, AppError> {
		let updated_user = db::users::update_user(&self.pool, user_id, dto)
		.await?
		.ok_or(AppError::NotFound(format!("User with id {} not found", user_id)))?;
		Ok(updated_user)
	}

	pub async fn delete_user(&self, user_id: i32) -> Result<bool, AppError> {
		let deleted = db::users::delete_user(&self.pool, user_id).await?;
		Ok(deleted)
	}
}