//! IPC command for offline dictionary lookup (FR-NOTE-03).
//!
//! The first lookup builds the dictionary index from the bundled data set, which
//! is heavy, so the work runs on a blocking thread to keep the async runtime and
//! the UI responsive. Subsequent lookups hit the cached index.

use crate::ipc::dto::DefinitionDto;
use crate::ipc::error::AppError;
use crate::state::AppState;

#[tauri::command]
pub async fn dictionary_lookup(
    state: tauri::State<'_, AppState>,
    word: String,
) -> Result<Vec<DefinitionDto>, AppError> {
    let dictionary = state.dictionary.clone();
    let definitions = tauri::async_runtime::spawn_blocking(move || dictionary.lookup(&word))
        .await
        .map_err(|e| AppError::from_internal(e.to_string()))??;
    Ok(definitions.into_iter().map(DefinitionDto::from).collect())
}
