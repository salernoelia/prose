import type { ReadingStyle } from './types'

// Theme token maps: background color and text color injected directly into the
// ePub iframe so the reading surface matches the app shell's active theme.
export const THEME_TOKENS = {
  light: { bg: '#F7EDDA', fg: '#1C1917' },
  paper: { bg: '#ffffff', fg: '#111111' },
  dark: { bg: '#09332C', fg: '#F7EDDA' },
  oled: { bg: '#000000', fg: '#F7EDDA' },
  night: { bg: '#000000', fg: '#8C7D70' },
  sepia: { bg: '#E4D7BE', fg: '#2E2218' },
  'sepia-dark': { bg: '#1C1611', fg: '#F7EDDA' },
  eink: { bg: '#ffffff', fg: '#000000' },
  'eink-dark': { bg: '#000000', fg: '#ffffff' },
} as const

/** Themes that render light text on a dark surface. */
export const DARK_THEMES = new Set<ReadingStyle['theme']>([
  'dark',
  'oled',
  'night',
  'sepia-dark',
  'eink-dark',
])

export function isDarkTheme(theme: ReadingStyle['theme']): boolean {
  return DARK_THEMES.has(theme)
}
