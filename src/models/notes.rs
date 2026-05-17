use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Note {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub content: String,
}
