/**
 * useReadingTracker — times active reading sessions and persists them through
 * the Rust core so they participate in WebDAV sync (streaks, weekly activity,
 * and total reading time stay consistent across devices).
 *
 * Call `startSession(book)` when a book is opened and `endSession()` when the
 * reader unmounts. Sessions shorter than MIN_SESSION_MS are discarded to avoid
 * noise from accidental taps. Persisted sessions are the atoms the statistics
 * view derives from; nothing aggregated is stored.
 */

import type { BookDto } from '../ipc/types'
import { readingLogSession } from '../ipc/reading'
import { refreshSessions } from '../stores/sessions'
import { syncState, triggerSync } from '../stores/sync'

const MIN_SESSION_MS = 10_000 // 10 seconds minimum to count

// Module-level so multiple calls share the same reference
let sessionStart: number | null = null
let activeBook: BookDto | null = null

export function startSession(book: BookDto): void {
  sessionStart = Date.now()
  activeBook = book
}

export function endSession(): void {
  if (!sessionStart || !activeBook) return

  const startedAt = sessionStart
  const elapsed = Date.now() - startedAt
  const book = activeBook
  sessionStart = null
  activeBook = null

  if (elapsed < MIN_SESSION_MS) return

  const durationSeconds = Math.round(elapsed / 1000)

  void (async () => {
    try {
      await readingLogSession(book.id, startedAt, durationSeconds)
      await refreshSessions()
      // Push the new session promptly, as the library does on import/remove.
      if (syncState.configured) void triggerSync()
    } catch (err) {
      console.error('Failed to log reading session:', err)
    }
  })()
}
