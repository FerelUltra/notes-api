use crate::{
    db,
    dto::notes::{CreateNoteDto, NoteResponseDto, UpdateNoteDto},
    errors::AppError,
    models::notes::Note,
    state::AppState,
};

#[derive(Clone)]
pub struct NoteService {
    state: AppState,
}

impl NoteService {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub async fn get_notes_by_user_id(
        &self,
        user_id: i32,
    ) -> Result<Vec<NoteResponseDto>, AppError> {
        let notes = db::notes::get_notes_by_user_id(&self.state.db, user_id).await?;

        let response = notes
            .into_iter()
            .map(
                |Note {
                     id, title, content, ..
                 }| NoteResponseDto { id, title, content },
            )
            .collect();

        Ok(response)
    }

    pub async fn create_note(
        &self,
        user_id: i32,
        dto: CreateNoteDto,
    ) -> Result<NoteResponseDto, AppError> {
        let note = db::notes::create_note(&self.state.db, user_id, dto).await?;

        Ok(note.into())
    }

    pub async fn get_note_by_id(
        &self,
        user_id: i32,
        note_id: i32,
    ) -> Result<NoteResponseDto, AppError> {
        let note = db::notes::get_note_by_id(&self.state.db, user_id, note_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Note with id {} not found", note_id)))?;

        Ok(note.into())
    }

    pub async fn update_note(
        &self,
        user_id: i32,
        note_id: i32,
        dto: UpdateNoteDto,
    ) -> Result<NoteResponseDto, AppError> {
        let note = db::notes::update_note(&self.state.db, user_id, note_id, dto)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Note with id {} not found", note_id)))?;

        Ok(note.into())
    }

    pub async fn delete_note(&self, user_id: i32, note_id: i32) -> Result<(), AppError> {
        let deleted = db::notes::delete_note(&self.state.db, user_id, note_id).await?;

        if !deleted {
            return Err(AppError::NotFound(format!(
                "Note with id {} not found",
                note_id
            )));
        }

        Ok(())
    }
}

impl From<Note> for NoteResponseDto {
    fn from(Note{ id, title, content, user_id: _user_id}: Note) -> Self {
        
        Self {
            id: id,
            title: title,
            content: content,
        }
    }
}
