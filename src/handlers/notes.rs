use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::{
    auth::extractor::AuthUser,
    dto::notes::{CreateNoteDto, NoteResponseDto, UpdateNoteDto},
    errors::AppError,
    services::notes::NoteService,
    state::AppState,
};

pub async fn get_notes(
    AuthUser { user_id }: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<NoteResponseDto>>, AppError> {
    let service = NoteService::new(state);

    let notes = service.get_notes_by_user_id(user_id).await?;

    Ok(Json(notes))
}

pub async fn create_note(
    AuthUser { user_id }: AuthUser,
    State(state): State<AppState>,
    Json(dto): Json<CreateNoteDto>,
) -> Result<(StatusCode, Json<NoteResponseDto>), AppError> {
    dto.validate()?;

    let service = NoteService::new(state);
    let note = service.create_note(user_id, dto).await?;

    Ok((StatusCode::CREATED, Json(note)))
}

pub async fn get_note_by_id(
    AuthUser { user_id }: AuthUser,
    State(state): State<AppState>,
    Path(note_id): Path<i32>,
) -> Result<Json<NoteResponseDto>, AppError> {
    let service = NoteService::new(state);
    let note = service.get_note_by_id(user_id, note_id).await?;

    Ok(Json(note))
}

pub async fn update_note(
    AuthUser { user_id }: AuthUser,
    State(state): State<AppState>,
    Path(note_id): Path<i32>,
    Json(dto): Json<UpdateNoteDto>,
) -> Result<Json<NoteResponseDto>, AppError> {
    dto.validate()?;

    let service = NoteService::new(state);
    let note = service.update_note(user_id, note_id, dto).await?;

    Ok(Json(note))
}

pub async fn delete_note(
    AuthUser { user_id }: AuthUser,
    State(state): State<AppState>,
    Path(note_id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let service = NoteService::new(state);
    service.delete_note(user_id, note_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
