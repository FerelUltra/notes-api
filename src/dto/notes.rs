use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct NoteResponseDto {
    pub id: i32,
    pub title: String,
    pub content: String,
}
