//! IPC commands for reading position: save and resume.
//!
//! Position is a format-neutral [`LocatorDto`]; the domain interprets only the
//! `progression` fraction. Saving position also emits `library:changed` so the
//! library view can update its progress display and last-read sort column.

use tauri::{AppHandle, Emitter};

use crate::domain::model::BookId;
use crate::ipc::dto::{LocatorDto, ProgressDto};
use crate::ipc::error::AppError;
use crate::ipc::event::LIBRARY_CHANGED;
use crate::state::AppState;

#[tauri::command]
pub async fn reading_save_position(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    book_id: String,
    locator: LocatorDto,
) -> Result<ProgressDto, AppError> {
    let id = BookId::from_hash(&book_id);
    let progress = state.reading.save_position(&id, locator.into())?;
    app.emit(LIBRARY_CHANGED, ())
        .map_err(|e| AppError::from_internal(e.to_string()))?;
    Ok(ProgressDto::from(progress))
}

#[tauri::command]
pub async fn reading_get_position(
    state: tauri::State<'_, AppState>,
    book_id: String,
) -> Result<Option<ProgressDto>, AppError> {
    let id = BookId::from_hash(&book_id);
    let progress = state.reading.get_position(&id)?;
    Ok(progress.map(ProgressDto::from))
}
