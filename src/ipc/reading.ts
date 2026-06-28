/**
 * Reading position commands: the only place `invoke` appears for reading state.
 *
 * Position is a format-neutral `LocatorDto`; the Rust core interprets only
 * the `progression` fraction for percentage display and sync conflict resolution.
 */
import { invoke } from '@tauri-apps/api/core'
import type { LocatorDto, ProgressDto } from './types'

/** Persist the current reading position for a book. Returns the saved progress. */
export function readingSavePosition(bookId: string, locator: LocatorDto): Promise<ProgressDto> {
  return invoke('reading_save_position', { bookId, locator })
}

/** Retrieve the last saved position for a book, or `null` if never opened. */
export function readingGetPosition(bookId: string): Promise<ProgressDto | null> {
  return invoke('reading_get_position', { bookId })
}
