import { computed, ref, watch } from 'vue'
import { settingsState, updateSettings, initSettingsStore } from '../stores/settings'
import type { Theme, TextAlign } from '../ipc/types'
import type { TranslationLanguage } from '../lib/externalLookup'
import { defaultTranslationLanguage } from '../lib/externalLookup'

// Shared reactive module-scoped state for click zone size, persisted locally
const clickZoneSize = ref(
  Number(
    (typeof localStorage !== 'undefined' ? localStorage.getItem('clickZoneSize') : null) || '25',
  ),
)
const showClickZonePreview = ref(false)

// Which language the reader's Translate action targets; a local view
// preference, stored alongside the click zone size rather than in the synced
// domain core. Defaults to the device language on first run.
const translationLanguage = ref<TranslationLanguage>(
  ((typeof localStorage !== 'undefined'
    ? localStorage.getItem('translationLanguage')
    : null) as TranslationLanguage | null) || defaultTranslationLanguage(),
)

let previewTimeout: ReturnType<typeof setTimeout> | null = null

// Watch clickZoneSize to temporarily display click zone visual overlays
watch(clickZoneSize, () => {
  showClickZonePreview.value = true
  if (previewTimeout) {
    clearTimeout(previewTimeout)
  }
  previewTimeout = setTimeout(() => {
    showClickZonePreview.value = false
  }, 1200)
})

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

  const textAlign = computed({
    get: () => settingsState.settings.textAlign,
    set: (value: TextAlign) => {
      updateSettings({ textAlign: value })
    },
  })

  const clickZone = computed({
    get: () => clickZoneSize.value,
    set: (value: number) => {
      clickZoneSize.value = value
      if (typeof localStorage !== 'undefined') {
        localStorage.setItem('clickZoneSize', String(value))
      }
    },
  })

  const translation = computed({
    get: () => translationLanguage.value,
    set: (value: TranslationLanguage) => {
      translationLanguage.value = value
      if (typeof localStorage !== 'undefined') {
        localStorage.setItem('translationLanguage', value)
      }
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
    textAlign,
    clickZoneSize: clickZone,
    translationLanguage: translation,
    showClickZonePreview,
    updateSettings,
  }
}
