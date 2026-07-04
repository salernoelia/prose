//! IPC commands for reading position: save and resume.
//!
//! Position is a format-neutral [`LocatorDto`]; the domain interprets only the
//! `progression` fraction. Saving position also emits `library:changed` so the
//! library view can update its progress display and last-read sort column.

use tauri::{AppHandle, Emitter};

use crate::domain::model::BookId;
use crate::ipc::dto::{LocatorDto, ProgressDto, ReadingSessionDto};
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

/// Record a finished reading session for a book. `started_at` is epoch
/// milliseconds; the calendar day used for streaks is derived from it on the
/// client. Returns the stored session.
#[tauri::command]
pub async fn reading_log_session(
    state: tauri::State<'_, AppState>,
    book_id: String,
    started_at: i64,
    duration_seconds: i64,
) -> Result<ReadingSessionDto, AppError> {
    let id = BookId::from_hash(&book_id);
    let session = state
        .reading
        .log_session(&id, started_at, duration_seconds)?;
    Ok(ReadingSessionDto::from(session))
}

/// Every recorded reading session across the library, newest first. The
/// statistics view derives all aggregates from these.
#[tauri::command]
pub async fn reading_list_sessions(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ReadingSessionDto>, AppError> {
    let sessions = state.reading.list_sessions()?;
    Ok(sessions.into_iter().map(ReadingSessionDto::from).collect())
}

/// Delete a recorded session, e.g. one logged by mistake. The core keeps a
/// tombstone so the deletion propagates through sync instead of the session
/// resurrecting from the remote copy.
#[tauri::command]
pub async fn reading_delete_session(
    state: tauri::State<'_, AppState>,
    session_id: String,
) -> Result<(), AppError> {
    state.reading.delete_session(&session_id)?;
    Ok(())
}
