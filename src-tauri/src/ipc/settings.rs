//! Settings commands: the `settings_get` and `settings_patch` handlers.
//!
//! These are the reference slice for the IPC pattern: a thin command calls
//! the domain service, converts the result to a DTO, and emits an event on
//! mutation. Every later command group follows this shape.

use tauri::AppHandle;
use tauri::Emitter;

use crate::ipc::dto::{SettingsDto, SettingsPatchDto};
use crate::ipc::error::AppError;
use crate::ipc::event::{SettingsChangedPayload, SETTINGS_CHANGED};
use crate::state::AppState;

#[tauri::command]
pub fn settings_get(state: tauri::State<'_, AppState>) -> Result<SettingsDto, AppError> {
    let settings = state.settings.get()?;
    Ok(SettingsDto::from(settings))
}

#[tauri::command]
pub fn settings_patch(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    patch: SettingsPatchDto,
) -> Result<SettingsDto, AppError> {
    let domain_patch = patch.try_into().map_err(AppError::from)?;
    let updated = state.settings.patch(&domain_patch)?;
    let dto = SettingsDto::from(updated);
    app.emit(
        SETTINGS_CHANGED,
        SettingsChangedPayload {
            settings: dto.clone(),
        },
    )
    .map_err(|e| AppError::from_internal(e.to_string()))?;
    Ok(dto)
}
