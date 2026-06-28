import { reactive, readonly } from 'vue'
import type { SettingsDto, SettingsPatchDto } from '../ipc/types'
import { getSettings, patchSettings } from '../ipc/settings'
import { onSettingsChanged } from '../ipc/events'

const defaults: SettingsDto = {
  schemaVersion: 1,
  theme: 'light',
  fontFamily: 'Literata',
  fontSize: 18,
  lineHeight: 1.5,
  margin: 1.0,
}

const state = reactive<{
  settings: SettingsDto
  loaded: boolean
}>({
  settings: { ...defaults },
  loaded: false,
})

let initPromise: Promise<void> | null = null

export function initSettingsStore(): Promise<void> {
  if (initPromise) return initPromise

  initPromise = (async () => {
    try {
      const stored = await getSettings()
      Object.assign(state.settings, stored)
      state.loaded = true
    } catch (err) {
      console.error('Failed to load settings from Rust core:', err)
    }

    try {
      await onSettingsChanged((payload) => {
        Object.assign(state.settings, payload.settings)
      })
    } catch (err) {
      console.error('Failed to subscribe to settings changed events:', err)
    }
  })()

  return initPromise
}

export const settingsState = readonly(state)

export async function updateSettings(patch: SettingsPatchDto): Promise<void> {
  try {
    const updated = await patchSettings(patch)
    Object.assign(state.settings, updated)
  } catch (err) {
    console.error('Failed to patch settings:', err)
    throw err
  }
}
