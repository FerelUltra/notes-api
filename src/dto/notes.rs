use crate::errors::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateNoteDto {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNoteDto {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct NoteResponseDto {
    pub id: i32,
    pub title: String,
    pub content: String,
}

const MAX_TITLE_SYMBOLS: usize = 200;
const CONTENT_MAX_SIZE: usize = 10_000;

fn validate_title(title: &str) -> Result<(), AppError> {
    let title = title.trim();

    if title.is_empty() {
        return Err(AppError::BadRequest("Title cannot be empty".to_string()));
    }

    if title.len() > MAX_TITLE_SYMBOLS {
        return Err(AppError::BadRequest("Title is too long".to_string()));
    }

    Ok(())
}

fn validate_content(content: &str) -> Result<(), AppError> {
    if content.len() > CONTENT_MAX_SIZE {
        return Err(AppError::BadRequest("Content is too long".to_string()));
    }

    Ok(())
}

impl CreateNoteDto {
    pub fn validate(&self) -> Result<(), AppError> {
        validate_title(&self.title)?;
        validate_content(&self.content)?;

        Ok(())
    }
}

impl UpdateNoteDto {
    pub fn validate(&self) -> Result<(), AppError> {
        validate_title(&self.title)?;
        validate_content(&self.content)?;

        Ok(())
    }
}
