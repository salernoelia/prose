/**
 * Reading sessions store: the reactive cache of the sync-backed session log.
 *
 * Sessions are the atoms the statistics view derives from (reading time,
 * streaks, charts). They live in the Rust/SQLite core and travel through
 * WebDAV sync exactly like progress and annotations, so this store only mirrors
 * them for the UI and refreshes when they may have changed (a finished sync or
 * a library mutation).
 */
import { reactive, readonly } from 'vue'
import type { ReadingSessionDto } from '../ipc/types'
import { readingListSessions, readingLogSession } from '../ipc/reading'
import { onLibraryChanged, onSyncFinished } from '../ipc/events'

const state = reactive<{
  sessions: ReadingSessionDto[]
  loaded: boolean
}>({
  sessions: [],
  loaded: false,
})

let initPromise: Promise<void> | null = null

export function initSessionsStore(): Promise<void> {
  if (initPromise) return initPromise

  initPromise = (async () => {
    await migrateLegacySessions()
    await refreshSessions()
    state.loaded = true

    try {
      // A finished sync may have pulled sessions from another device.
      await onSyncFinished(async (payload) => {
        if (payload.success) await refreshSessions()
      })
    } catch (err) {
      console.error('Failed to listen to sync:finished for sessions:', err)
    }

    try {
      // Removing a book cascade-deletes its sessions in the core.
      await onLibraryChanged(async () => {
        await refreshSessions()
      })
    } catch (err) {
      console.error('Failed to listen to library:changed for sessions:', err)
    }
  })()

  return initPromise
}

export async function refreshSessions(): Promise<void> {
  try {
    state.sessions = await readingListSessions()
  } catch (err) {
    console.error('Failed to load reading sessions:', err)
  }
}

export const sessionsState = readonly(state)

// ── Legacy migration ─────────────────────────────────────────────────────────

const LEGACY_KEY = 'prose_reading_sessions'
const MIGRATED_FLAG = 'prose_reading_sessions_migrated'

interface LegacySession {
  bookId: string
  date: string // YYYY-MM-DD local
  durationSeconds: number
}

/**
 * One-time import of sessions previously kept in localStorage into the
 * sync-backed core. Legacy records only carried a calendar day, so each is
 * stamped at local noon of that day, close enough for streaks and totals.
 * Best-effort: a session whose book is no longer in the library is skipped.
 */
async function migrateLegacySessions(): Promise<void> {
  try {
    if (localStorage.getItem(MIGRATED_FLAG)) return
    const raw = localStorage.getItem(LEGACY_KEY)
    if (!raw) {
      localStorage.setItem(MIGRATED_FLAG, '1')
      return
    }

    const legacy = JSON.parse(raw) as LegacySession[]
    for (const s of legacy) {
      const [y, m, d] = s.date.split('-').map(Number)
      const startedAt = new Date(y, m - 1, d, 12, 0, 0, 0).getTime()
      try {
        await readingLogSession(s.bookId, startedAt, s.durationSeconds)
      } catch {
        // Book gone or invalid record: drop it and continue.
      }
    }

    localStorage.setItem(MIGRATED_FLAG, '1')
    localStorage.removeItem(LEGACY_KEY)
  } catch (err) {
    console.error('Failed to migrate legacy reading sessions:', err)
  }
}
