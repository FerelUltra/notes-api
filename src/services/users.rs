use sqlx::PgPool;

use crate::{
    auth::{hash_password, verify_password}, db, dto::{
        RegisterUserDto, users::{CreateUserDto, UpdateUserDto}, LoginUserDto
    }, errors::AppError, models::users::User
};

#[derive(Clone)]
pub struct UserService {
    pool: PgPool,
}

impl UserService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn register_user(&self, dto: RegisterUserDto) -> Result<User, AppError> {
        // 1. validate (can be reused later)
        if dto.name.trim().is_empty() {
            return Err(AppError::BadRequest("Name cannot be empty".into()));
        }

        if dto.password.len() < 6 {
            return Err(AppError::BadRequest("Password too short".into()));
        }

        // 2. hash password
        let password_hash = hash_password(&dto.password)?;

        // 3. save user
        let user = db::users::create_user_with_password(
            &self.pool,
            dto.name,
            password_hash,
        ).await?;

        Ok(user)
    }

    pub async fn create_user(&self, dto: CreateUserDto) -> Result<User, AppError> {
        let created_user = db::users::create_user(&self.pool, dto).await?;
        Ok(created_user)
    }

    pub async fn get_user_by_id(&self, user_id: i32) -> Result<User, AppError> {
        let user = db::users::get_user_by_id(&self.pool, user_id)
            .await?
            .ok_or(AppError::NotFound(format!(
                "User with id {} not found",
                user_id
            )))?;
        Ok(user)
    }

    pub async fn get_users(&self) -> Result<Vec<User>, AppError> {
        let users = db::users::get_users(&self.pool).await?;
        Ok(users)
    }

    pub async fn update_user(&self, user_id: i32, dto: UpdateUserDto) -> Result<User, AppError> {
        let updated_user = db::users::update_user(&self.pool, user_id, dto)
            .await?
            .ok_or(AppError::NotFound(format!(
                "User with id {} not found",
                user_id
            )))?;
        Ok(updated_user)
    }

    pub async fn delete_user(&self, user_id: i32) -> Result<bool, AppError> {
        let deleted = db::users::delete_user(&self.pool, user_id).await?;
        Ok(deleted)
    }

    pub async fn login_user(
        &self,
        dto: LoginUserDto,
    ) -> Result<User, AppError> {
        let record = db::users::get_user_by_name(&self.pool, &dto.name)
            .await?
            .ok_or(AppError::BadRequest("Invalid credentials".to_string()))?;

        let (user, password_hash) = record;

        let valid = verify_password(&dto.password, &password_hash)?;

        if !valid{
            return Err(AppError::BadRequest("Invalid credentials".to_string()));
        }

        Ok(user)
    }
}
