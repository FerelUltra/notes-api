use serde::Serialize;
use sqlx::FromRow;

#[derive(Serialize, FromRow, Clone)]
pub struct User {
	pub id: i32,
	pub name: String,
}