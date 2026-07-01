import type { ReadingStyle } from './types'

// Theme token maps: background color and text color injected directly into the
// ePub iframe so the reading surface matches the app shell's active theme.
export const THEME_TOKENS = {
  light: { bg: '#faf9f5', fg: '#18181b' },
  paper: { bg: '#ffffff', fg: '#111111' },
  dark: { bg: '#121212', fg: '#e4e4e7' },
  oled: { bg: '#000000', fg: '#f4f4f5' },
  // Night: OLED black with dim, warm text for low-light reading (issue #9).
  night: { bg: '#000000', fg: '#706862' },
  sepia: { bg: '#d0b580', fg: '#36291b' },
  'sepia-dark': { bg: '#1c1611', fg: '#e0cdab' },
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
