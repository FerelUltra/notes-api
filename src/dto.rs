use serde::Deserialize;
use crate::errors::AppError;

#[derive(Debug, Deserialize)]
pub struct CreateUserDto {
	pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserDto{
	pub name: String,
}

fn validate_name(name: &str) -> Result<(), AppError> {
	let trimmed_name = name.trim();

	if trimmed_name.is_empty(){
		return Err(AppError::BadRequest("Name cannot be empty".to_string()));
	}

	if trimmed_name.len() > 100 {
		return Err(AppError::BadRequest("Name is too long".to_string()));
	}

	Ok(())
}

impl CreateUserDto {
	pub fn validate(&self) -> Result<(), AppError> {
		validate_name(&self.name)
	}
}

impl UpdateUserDto {
	pub fn validate(&self) -> Result<(), AppError> {
		validate_name(&self.name)
	}
}