use axum::{extract::State, Json};

use crate::{
    auth::extractor::AuthUser, dto::notes::NoteResponseDto, errors::AppError,
    services::notes::NoteService, state::AppState,
};

pub async fn get_notes(
    AuthUser { user_id }: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<NoteResponseDto>>, AppError> {
    let service = NoteService::new(state);

    let notes = service.get_notes_by_user_id(user_id).await?;

    Ok(Json(notes))
}
