/**
 * Settings command wrappers: the only place `invoke` appears for settings.
 *
 * Each function mirrors a Rust `#[tauri::command]` in
 * `src-tauri/src/ipc/settings.rs`. Components call these through the settings
 * store or composable, never directly.
 */
import { invoke } from '@tauri-apps/api/core'
import type { SettingsDto, SettingsPatchDto } from './types'

/** Fetch the current settings from the Rust authority. */
export function getSettings(): Promise<SettingsDto> {
  return invoke<SettingsDto>('settings_get')
}

/** Send a partial settings update. Only the present fields change. */
export function patchSettings(patch: SettingsPatchDto): Promise<SettingsDto> {
  return invoke<SettingsDto>('settings_patch', { patch })
}
