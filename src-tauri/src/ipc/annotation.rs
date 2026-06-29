//! IPC commands for annotations: bookmarks and highlights (FR-NOTE-01/02).
//!
//! Thin wrappers over [`AnnotationService`]. The renderer supplies an opaque
//! [`LocatorDto`] for the location; highlights additionally carry the selected
//! `text` so they survive re-pagination. Each annotation is minted with a
//! stable, content-derived id in the domain, so sync stays idempotent.
//!
//! [`AnnotationService`]: crate::domain::AnnotationService

use crate::domain::model::BookId;
use crate::ipc::dto::{BookmarkDto, HighlightDto, LocatorDto};
use crate::ipc::error::AppError;
use crate::state::AppState;

#[tauri::command]
pub async fn annotation_add_bookmark(
    state: tauri::State<'_, AppState>,
    book_id: String,
    locator: LocatorDto,
) -> Result<BookmarkDto, AppError> {
    let id = BookId::from_hash(&book_id);
    let bookmark = state.annotations.add_bookmark(&id, locator.into())?;
    Ok(BookmarkDto::from(bookmark))
}

#[tauri::command]
pub async fn annotation_list_bookmarks(
    state: tauri::State<'_, AppState>,
    book_id: String,
) -> Result<Vec<BookmarkDto>, AppError> {
    let id = BookId::from_hash(&book_id);
    let bookmarks = state.annotations.list_bookmarks(&id)?;
    Ok(bookmarks.into_iter().map(BookmarkDto::from).collect())
}

#[tauri::command]
pub async fn annotation_delete_bookmark(
    state: tauri::State<'_, AppState>,
    bookmark_id: String,
) -> Result<(), AppError> {
    state.annotations.delete_bookmark(&bookmark_id)?;
    Ok(())
}

#[tauri::command]
pub async fn annotation_add_highlight(
    state: tauri::State<'_, AppState>,
    book_id: String,
    locator: LocatorDto,
    text: String,
    color: Option<String>,
) -> Result<HighlightDto, AppError> {
    let id = BookId::from_hash(&book_id);
    let highlight = state
        .annotations
        .add_highlight(&id, locator.into(), text, color)?;
    Ok(HighlightDto::from(highlight))
}

#[tauri::command]
pub async fn annotation_list_highlights(
    state: tauri::State<'_, AppState>,
    book_id: String,
) -> Result<Vec<HighlightDto>, AppError> {
    let id = BookId::from_hash(&book_id);
    let highlights = state.annotations.list_highlights(&id)?;
    Ok(highlights.into_iter().map(HighlightDto::from).collect())
}

#[tauri::command]
pub async fn annotation_delete_highlight(
    state: tauri::State<'_, AppState>,
    highlight_id: String,
) -> Result<(), AppError> {
    state.annotations.delete_highlight(&highlight_id)?;
    Ok(())
}
