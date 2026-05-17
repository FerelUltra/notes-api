use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct User {
    pub id: i32,
    pub name: String,
    #[serde(skip_serializing)]
    pub password_hash: Option<String>,
    pub email: Option<String>,
}
