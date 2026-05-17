use crate::errors::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RegisterUserDto {
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginUserDto {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponseDto {
    pub access_token: String,
    pub token: String,
}

const MAX_PASSWORD_LENGTH: usize = 128;
const MIN_PASSWORD_LENGTH: usize = 8;

const MAX_NAME_LENGTH: usize = 100;

const MAX_EMAIL_LENGTH: usize = 255;

impl RegisterUserDto {
    pub fn validate(&self) -> Result<(), AppError> {
        validate_name(&self.name)?;
        validate_email(&self.email)?;
        validate_password(&self.password)?;

        Ok(())
    }
}

impl LoginUserDto {
    pub fn validate(&self) -> Result<(), AppError> {
        validate_email(&self.email)?;

        if self.password.is_empty() {
            return Err(AppError::BadRequest("Password is required".to_string()));
        }

        Ok(())
    }
}

fn validate_name(name: &str) -> Result<(), AppError> {
    let name = name.trim();

    if name.is_empty() {
        return Err(AppError::BadRequest("Name cannot be empty".to_string()));
    }
    if name.len() > MAX_NAME_LENGTH {
        return Err(AppError::BadRequest("Name is too long".to_string()));
    }

    Ok(())
}

fn validate_email(email: &str) -> Result<(), AppError> {
    let email = email.trim();

    if email.is_empty() {
        return Err(AppError::BadRequest("Email is required".to_string()));
    }
    if email.len() >= MAX_EMAIL_LENGTH {
        return Err(AppError::BadRequest("Email is too long".to_string()));
    }

    if !email.contains('@') {
        return Err(AppError::BadRequest("Email is invalid".to_string()));
    }

    let parts: Vec<&str> = email.split('@').collect();

    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(AppError::BadRequest("Email is invalid".to_string()));
    }

    if !parts[1].contains('.') {
        return Err(AppError::BadRequest("Email is invalid".to_string()));
    }

    Ok(())
}

fn validate_password(password: &str) -> Result<(), AppError> {
    if password.len() < MIN_PASSWORD_LENGTH {
        return Err(AppError::BadRequest(
            "Password must be at least 8 characters".to_string(),
        ));
    }

    if password.len() > MAX_PASSWORD_LENGTH {
        return Err(AppError::BadRequest("Password is too long".to_string()));
    }

    Ok(())
}
