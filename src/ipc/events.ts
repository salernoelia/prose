/**
 * Typed event listeners: the only place `listen` appears.
 *
 * Each function wraps a Tauri event with the matching payload type from
 * `src/ipc/types.ts`. Stores call these on startup; components never
 * listen to raw events.
 */
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { EventNames, type SettingsChangedPayload } from './types'

/** Subscribe to settings changes pushed from the Rust core. */
export function onSettingsChanged(
  callback: (payload: SettingsChangedPayload) => void,
): Promise<UnlistenFn> {
  return listen<SettingsChangedPayload>(EventNames.SETTINGS_CHANGED, (event) =>
    callback(event.payload),
  )
}
