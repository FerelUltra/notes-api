use sqlx::PgPool;

use crate::{
    auth::{hash_password, verify_password},
    db,
    dto::{users::UpdateUserDto, LoginUserDto, RegisterUserDto},
    errors::AppError,
    models::users::User,
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
        
        let existing_user = db::users::get_user_by_email(&self.pool, &dto.email).await?;

        if existing_user.is_some() {
            return Err(AppError::Conflict("Email already exists".to_string()))
        }

        // 2. hash password
        let password_hash = hash_password(&dto.password)?;

        // 3. save user
        let user = db::users::create_user(
            &self.pool,
            db::users::CreateUserDb {
                name: dto.name,
                email: dto.email,
                password_hash,
            },
        )
        .await?;

        Ok(user)
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

    pub async fn login_user(&self, dto: LoginUserDto) -> Result<User, AppError> {
        let user = db::users::get_user_by_email(&self.pool, &dto.email)
            .await?
            .ok_or(AppError::Unauthorized(
                "Invalid email or password".to_string(),
            ))?;

        let is_valid = verify_password(
            &dto.password,
            user.password_hash
                .as_deref()
                .ok_or_else(|| AppError::Unauthorized("Invalid email or password".to_string()))?,
        )?;

        if !is_valid {
            return Err(AppError::Unauthorized(
                "Invalid email or password".to_string(),
            ));
        }

        Ok(user)
    }
}
