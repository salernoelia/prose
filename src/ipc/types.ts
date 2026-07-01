/**
 * Hand-mirrored TypeScript types for the Rust IPC boundary.
 *
 * These types correspond 1:1 with the DTOs in `src-tauri/src/ipc/dto.rs`,
 * the error in `src-tauri/src/ipc/error.rs`, and the event payloads in
 * `src-tauri/src/ipc/event.rs`. They are the **single source of truth** for
 * the TypeScript side of the boundary (architecture section 4.5, strategy 1).
 *
 * Rules:
 * - Every field is camelCase, matching the Rust `#[serde(rename_all = "camelCase")]`.
 * - Keep this file in sync with the Rust DTOs during review.
 * - When the command surface grows, consider adopting `tauri-specta` to
 *   generate these types automatically.
 */

// Error

/** The serializable error returned by every Tauri command. */
export interface AppError {
  code: string
  message: string
}

// Settings

/** The full, flattened settings as returned by `settings_get`. */
export interface SettingsDto {
  schemaVersion: number
  theme: Theme
  fontFamily: string
  fontSize: number
  lineHeight: number
  margin: number
}

/** Partial settings update sent by `settings_patch`. Only present fields change. */
export interface SettingsPatchDto {
  theme?: Theme
  fontFamily?: string
  fontSize?: number
  lineHeight?: number
  margin?: number
}

/** The supported reading themes. */
export type Theme =
  | 'light'
  | 'paper'
  | 'dark'
  | 'oled'
  | 'night'
  | 'sepia'
  | 'sepia-dark'
  | 'eink'
  | 'eink-dark'

// Event payloads

/** Payload for the `settings:changed` event. */
export interface SettingsChangedPayload {
  settings: SettingsDto
}

/** Payload for the `import:progress` event. */
export interface ImportProgressPayload {
  message: string
  fraction: number
}

/** Payload for the `sync:progress` event. */
export interface SyncProgressPayload {
  stage: string
  fraction: number
}

/** Payload for the `sync:finished` event. */
export interface SyncFinishedPayload {
  success: boolean
  message: string
}

// Library

export type Format = 'epub' | 'pdf'

export interface BookDto {
  id: string
  format: Format
  title: string
  author: string | null
  cover: string | null
}

export interface LibraryEntryDto {
  book: BookDto
  progress: number
  lastRead: number | null
}

export type SortKey = 'title' | 'author' | 'last_read' | 'progress'

export interface LibraryQueryDto {
  search: string | null
  sort: SortKey
  descending: boolean
}

// Reading

/** Format-neutral reading position, mirroring the Rust `LocatorDto`. */
export interface LocatorDto {
  payload: string
  progression: number
}

/** Saved reading position with its timestamp, returned by position commands. */
export interface ProgressDto {
  locator: LocatorDto
  updatedAt: number
}

// Annotations

/** A bookmark at a saved location, mirroring the Rust `BookmarkDto`. */
export interface BookmarkDto {
  id: string
  bookId: string
  locator: LocatorDto
  createdAt: number
}

/** A highlight over a selected text range, mirroring the Rust `HighlightDto`. */
export interface HighlightDto {
  id: string
  bookId: string
  locator: LocatorDto
  text: string
  color: string | null
  createdAt: number
}

// Reading sessions

/**
 * A reading session, mirroring the Rust `ReadingSessionDto`. The calendar day
 * used for streaks is derived from `startedAt` in local time; book title and
 * author are looked up from the library. No aggregate (reading time, streaks,
 * charts) is stored, all are derived from these records.
 */
export interface ReadingSessionDto {
  id: string
  bookId: string
  startedAt: number
  durationSeconds: number
}

// Dictionary

/** One dictionary sense, mirroring the Rust `DefinitionDto` (FR-NOTE-03). */
export interface DefinitionDto {
  partOfSpeech: string
  gloss: string
  synonyms: string[]
  examples: string[]
}

// Sync

/** Current sync configuration (no secrets). */
export interface SyncStatusDto {
  configured: boolean
  url: string | null
  username: string | null
}

/** A book file available on the remote server. */
export interface RemoteBookDto {
  path: string
  etag: string | null
}

// Event name constants

/** Mirrors the Rust `ipc::event` constants so listeners use the same strings. */
export const EventNames = {
  SETTINGS_CHANGED: 'settings:changed',
  LIBRARY_CHANGED: 'library:changed',
  IMPORT_PROGRESS: 'import:progress',
  SYNC_PROGRESS: 'sync:progress',
  SYNC_FINISHED: 'sync:finished',
} as const
