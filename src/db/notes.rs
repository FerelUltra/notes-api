use sqlx::PgPool;

use crate::{
    dto::notes::{CreateNoteDto, UpdateNoteDto},
    errors::AppError,
    models::notes::Note,
};

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

pub async fn create_note(
    pool: &PgPool,
    user_id: i32,
    dto: CreateNoteDto,
) -> Result<Note, AppError> {
    let note = sqlx::query_as::<_, Note>(
        r#"
        INSERT INTO notes (user_id, title, content)
        VALUES ($1, $2, $3)
        RETURNING id, user_id, title, content
        "#,
    )
    .bind(user_id)
    .bind(dto.title.trim())
    .bind(dto.content)
    .fetch_one(pool)
    .await?;

    Ok(note)
}

pub async fn get_note_by_id(
    pool: &PgPool,
    user_id: i32,
    note_id: i32,
) -> Result<Option<Note>, AppError> {
    let note = sqlx::query_as::<_, Note>(
        r#"
        SELECT id, user_id, title, content
        FROM notes
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(note_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(note)
}

pub async fn update_note(
    pool: &PgPool,
    user_id: i32,
    note_id: i32,
    dto: UpdateNoteDto,
) -> Result<Option<Note>, AppError> {
    let note = sqlx::query_as::<_, Note>(
        r#"
        UPDATE notes
        SET title = $1, content = $2
        WHERE id = $3 AND user_id = $4
        RETURNING id, user_id, title, content
        "#,
    )
    .bind(dto.title.trim())
    .bind(dto.content)
    .bind(note_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(note)
}

pub async fn delete_note(pool: &PgPool, user_id: i32, note_id: i32) -> Result<bool, AppError> {
    let result = sqlx::query(
        r#"
        DELETE FROM notes
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(note_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
