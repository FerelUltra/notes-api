use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RegisterUserDto{
    pub name: String,
    pub password: String
}

#[derive(Debug, Deserialize)]
pub struct LoginUserDto {
    pub name: String,
    pub password: String
}