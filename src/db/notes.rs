use sqlx::PgPool;

use crate::{errors::AppError, models::notes::Note};

pub async fn get_notes_by_user_id(pool: &PgPool, user_id: i32) -> Result<Vec<Note>, AppError> {
    let notes = sqlx::query_as::<_, Note>(
        r#"
        SELECT id, user_id, title, content
        FROM notes
        WHERE user_id = $1
        ORDER BY id
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(notes)
}
