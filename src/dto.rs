use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateUserDto {
	pub name: String,
}

#[derive(Deserialize)]
pub struct UpdateUserDto{
	pub name: String,
}