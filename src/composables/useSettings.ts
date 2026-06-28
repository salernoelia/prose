import { computed } from 'vue'
import { settingsState, updateSettings, initSettingsStore } from '../stores/settings'
import type { Theme } from '../ipc/types'

export function useSettings() {
  initSettingsStore()

  const settings = computed(() => settingsState.settings)
  const loaded = computed(() => settingsState.loaded)

  const theme = computed({
    get: () => settingsState.settings.theme,
    set: (value: Theme) => {
      updateSettings({ theme: value })
    },
  })

  const fontFamily = computed({
    get: () => settingsState.settings.fontFamily,
    set: (value: string) => {
      updateSettings({ fontFamily: value })
    },
  })

  const fontSize = computed({
    get: () => settingsState.settings.fontSize,
    set: (value: number) => {
      updateSettings({ fontSize: value })
    },
  })

  const lineHeight = computed({
    get: () => settingsState.settings.lineHeight,
    set: (value: number) => {
      updateSettings({ lineHeight: value })
    },
  })

  const margin = computed({
    get: () => settingsState.settings.margin,
    set: (value: number) => {
      updateSettings({ margin: value })
    },
  })

  return {
    settings,
    loaded,
    theme,
    fontFamily,
    fontSize,
    lineHeight,
    margin,
    updateSettings,
  }
}
