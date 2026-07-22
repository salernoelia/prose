/**
 * useReadingTracker - times active reading sessions and persists them through
 * the Rust core so they participate in WebDAV sync (streaks, weekly activity,
 * and total reading time stay consistent across devices).
 *
 * Call `startSession(book)` when a book is opened and `endSession()` when the
 * reader unmounts. Sessions shorter than MIN_SESSION_MS are discarded to avoid
 * noise from accidental taps. Persisted sessions are the atoms the statistics
 * view derives from; nothing aggregated is stored.
 *
 * Only foreground time counts. A book left open while the app is backgrounded
 * or the device sleeps must not inflate the session, so time accrues in small
 * increments: the clock pauses while the document is hidden, and a wall-clock
 * gap larger than MAX_GAP_MS (timers stall across system sleep, so the gap
 * surfaces at the next tick) is discarded rather than counted.
 */

import type { BookDto } from '../ipc/types'
import { readingLogSession } from '../ipc/reading'
import { refreshSessions } from '../stores/sessions'
import { syncState, triggerSync } from '../stores/sync'

const MIN_SESSION_MS = 10_000 // 10 seconds minimum to count
const HEARTBEAT_MS = 30_000
// Anything beyond this between two ticks is sleep or a stalled WebView, not reading.
const MAX_GAP_MS = 60_000

// Module-level so multiple calls share the same reference
let sessionStart: number | null = null
let activeBook: BookDto | null = null
let accumulatedMs = 0
let lastTick = 0
let heartbeat: ReturnType<typeof setInterval> | null = null

/** Bank the time since the last tick, unless the gap says we were not reading. */
function accumulate(): void {
  const now = Date.now()
  const elapsed = now - lastTick
  if (elapsed > 0 && elapsed <= MAX_GAP_MS) accumulatedMs += elapsed
  lastTick = now
}

function startHeartbeat(): void {
  if (heartbeat) return
  heartbeat = setInterval(accumulate, HEARTBEAT_MS)
}

function stopHeartbeat(): void {
  if (heartbeat) {
    clearInterval(heartbeat)
    heartbeat = null
  }
}

function onVisibilityChange(): void {
  if (!sessionStart) return
  if (document.visibilityState === 'hidden') {
    accumulate()
    stopHeartbeat()
  } else {
    lastTick = Date.now()
    startHeartbeat()
  }
}

export function startSession(book: BookDto): void {
  sessionStart = Date.now()
  activeBook = book
  accumulatedMs = 0
  lastTick = sessionStart
  document.addEventListener('visibilitychange', onVisibilityChange)
  if (document.visibilityState !== 'hidden') startHeartbeat()
}

export function endSession(): void {
  if (!sessionStart || !activeBook) return

  // While hidden the clock is already paused; only bank the final stretch when
  // the session ends in the foreground.
  if (heartbeat) accumulate()
  stopHeartbeat()
  document.removeEventListener('visibilitychange', onVisibilityChange)

  const startedAt = sessionStart
  const elapsed = accumulatedMs
  const book = activeBook
  sessionStart = null
  activeBook = null
  accumulatedMs = 0

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
