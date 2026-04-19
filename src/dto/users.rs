use serde::{Deserialize, Serialize};
use crate::errors::AppError;

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest{
	pub username: String,
	pub email: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateUserRequest{
	pub username: String,
	pub email: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
	pub id: i64,
	pub username: String, 
	pub email: String,
}


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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn create_user_dto_validate_should_fail_when_name_is_empty(){
		let dto = CreateUserDto{
			name: "".to_string(),
		};

		let result = dto.validate();

		assert!(result.is_err());
	}

	#[test]
	fn create_user_dto_validate_should_fail_when_name_is_only_spaces(){
		let dto = CreateUserDto {
			name: "     ".to_string(),
		};

		let result = dto.validate();

		assert!(result.is_err());
	}

	#[test]
	fn create_user_dto_validate_should_pass_when_name_is_valid(){
		let dto = CreateUserDto{
			name: "Ferel".to_string(),
		};

		let result = dto.validate();

		assert!(result.is_ok());
	}

	#[test]
	fn create_user_dto_validate_should_fail_when_name_is_too_long(){
		let dto = CreateUserDto{
			name: "a".repeat(101),
		};

		let result = dto.validate();

		assert!(result.is_err());
	}

	#[test]
	fn create_user_dto_validate_should_return_bad_request_for_empty_name(){
		let dto = CreateUserDto{
			name: "".to_string(),
		};

		let result = dto.validate();

		match result {
			Err(AppError::BadRequest(message)) =>{
				assert_eq!(message, "Name cannot be empty");
			}
			_ => panic!("expected AppError::BadRequest") 
		}
	}

	#[test]
	fn update_user_dto_validate_should_pass_when_name_is_valid(){
		let dto = UpdateUserDto{
			name: "Alice".to_string(),
		};

		let result = dto.validate();

		assert!(result.is_ok());
	}
}