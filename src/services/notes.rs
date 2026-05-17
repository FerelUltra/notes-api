use crate::{
    db, dto::notes::NoteResponseDto, errors::AppError, models::notes::Note, state::AppState,
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
}
