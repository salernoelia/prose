// The IPC boundary: the only place `invoke` and `listen` appear. Typed command
// wrappers and event listeners are re-exported here as each feature lands.
export { getSettings, patchSettings } from './settings'
export { onSettingsChanged } from './events'
export type {
  SettingsDto,
  SettingsPatchDto,
  SettingsChangedPayload,
  Theme,
} from './types'
