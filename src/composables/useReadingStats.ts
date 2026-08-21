/**
 * useReadingStats - derives reading statistics from two sources:
 *   1. The library store (book count, progress, lastRead timestamps)
 *   2. The sync-backed reading session log (sessions store)
 *
 * All computed values are reactive and recalculate when either store updates.
 * Sessions carry only a start instant and a duration; the calendar day and the
 * book title/author are derived here, never stored, so nothing redundant syncs.
 */

import { computed } from 'vue'
import { libraryState, initLibraryStore } from '../stores/library'
import { sessionsState, initSessionsStore } from '../stores/sessions'

/** A session normalized for the stats math: day plus book metadata resolved. */
interface ReadingSession {
  bookId: string
  bookTitle: string
  bookAuthor: string | null
  /** ISO date string YYYY-MM-DD in local time, derived from startedAt */
  date: string
  durationSeconds: number
}

export interface DayActivity {
  /** ISO date YYYY-MM-DD */
  date: string
  /** Total seconds read on this day */
  totalSeconds: number
  /** Display label e.g. "Mon" */
  label: string
}

export interface BookActivity {
  bookId: string
  bookTitle: string
  bookAuthor: string | null
  totalSeconds: number
}

/** One row of the session history list: a raw session with its book resolved. */
export interface SessionEntry {
  id: string
  bookTitle: string
  startedAt: number
  durationSeconds: number
}

export interface ChartPoint {
  /** ISO date label (YYYY-MM-DD for daily, week-start for weekly) */
  date: string
  /** Total seconds in this bucket */
  seconds: number
  /** Whether this bucket contains today */
  isToday: boolean
}

function isoToDate(iso: string): Date {
  // Parse YYYY-MM-DD as local date (not UTC)
  const [y, m, d] = iso.split('-').map(Number)
  return new Date(y, m - 1, d)
}

function dateToISO(d: Date): string {
  const y = d.getFullYear()
  const m = String(d.getMonth() + 1).padStart(2, '0')
  const day = String(d.getDate()).padStart(2, '0')
  return `${y}-${m}-${day}`
}

function todayISO(): string {
  return dateToISO(new Date())
}

/** Returns the ISO dates for the current week (Mon → Sun). */
function currentWeekDates(): string[] {
  const now = new Date()
  // JS getDay() is 0=Sun; we want Mon=0
  const dow = (now.getDay() + 6) % 7
  const monday = new Date(now)
  monday.setDate(now.getDate() - dow)
  monday.setHours(0, 0, 0, 0)

  const days: string[] = []
  for (let i = 0; i < 7; i++) {
    const d = new Date(monday)
    d.setDate(monday.getDate() + i)
    days.push(dateToISO(d))
  }
  return days
}

const DAY_LABELS = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun']

export function useReadingStats() {
  initLibraryStore()
  initSessionsStore()

  const entries = computed(() => libraryState.entries)

  // ── Library-derived stats ──────────────────────────────────────────────────

  const totalBooks = computed(() => entries.value.length)

  const booksFinished = computed(
    () => entries.value.filter((e) => e.progress >= 1).length,
  )

  const booksInProgress = computed(
    () => entries.value.filter((e) => e.progress > 0 && e.progress < 1).length,
  )

  const booksUnstarted = computed(
    () => entries.value.filter((e) => e.progress === 0).length,
  )

  const averageProgress = computed(() => {
    if (entries.value.length === 0) return 0
    const sum = entries.value.reduce((acc, e) => acc + e.progress, 0)
    return sum / entries.value.length
  })

  /** Most recently opened book entry (by lastRead timestamp). */
  const lastReadEntry = computed(() => {
    return entries.value
      .filter((e) => e.lastRead !== null)
      .sort((a, b) => (b.lastRead ?? 0) - (a.lastRead ?? 0))[0] ?? null
  })

  // ── Session-derived stats ──────────────────────────────────────────────────

  /**
   * Sessions normalized for the math below: the calendar day is derived from
   * each session's start instant in local time, and the book title/author are
   * resolved from the library (falling back gracefully when a book was removed).
   */
  const sessions = computed<ReadingSession[]>(() => {
    const byId = new Map(entries.value.map((e) => [e.book.id, e.book]))
    return sessionsState.sessions.map((s) => {
      const book = byId.get(s.bookId)
      return {
        bookId: s.bookId,
        bookTitle: book?.title ?? 'Unknown book',
        bookAuthor: book?.author ?? null,
        date: dateToISO(new Date(s.startedAt)),
        durationSeconds: s.durationSeconds,
      }
    })
  })

  const totalReadingSeconds = computed(() =>
    sessions.value.reduce((acc, s) => acc + s.durationSeconds, 0),
  )

  /**
   * Raw sessions for the history list, newest first (the store keeps that
   * order). Unlike the aggregates above these carry the session id, so a row
   * can be deleted, e.g. one logged by mistake.
   */
  const sessionHistory = computed<SessionEntry[]>(() => {
    const byId = new Map(entries.value.map((e) => [e.book.id, e.book]))
    return sessionsState.sessions.map((s) => ({
      id: s.id,
      bookTitle: byId.get(s.bookId)?.title ?? 'Unknown book',
      startedAt: s.startedAt,
      durationSeconds: s.durationSeconds,
    }))
  })

  /** Map of ISO date → total seconds read on that date. */
  const sessionsByDate = computed(() => {
    const map = new Map<string, number>()
    for (const s of sessions.value) {
      map.set(s.date, (map.get(s.date) ?? 0) + s.durationSeconds)
    }
    return map
  })

  /** Current reading streak (consecutive calendar days up to today with ≥1 session). */
  const currentStreak = computed(() => {
    if (sessions.value.length === 0) return 0

    const today = todayISO()
    const uniqueDates = [...new Set(sessions.value.map((s) => s.date))].sort().reverse()

    let streak = 0
    const cursor = isoToDate(today)

    for (let i = 0; i < uniqueDates.length; i++) {
      const cursorISO = dateToISO(cursor)

      // Allow today to count even if no session yet (we haven't read today but
      // the streak is still alive from yesterday)
      if (uniqueDates[i] === cursorISO || (i === 0 && uniqueDates[i] !== cursorISO && cursorISO === today)) {
        if (uniqueDates[i] === cursorISO) {
          streak++
          cursor.setDate(cursor.getDate() - 1)
        } else {
          // today not read yet - check yesterday
          cursor.setDate(cursor.getDate() - 1)
          if (uniqueDates[i] === dateToISO(cursor)) {
            streak++
            cursor.setDate(cursor.getDate() - 1)
          } else {
            break
          }
        }
      } else {
        break
      }
    }

    return streak
  })

  /** Best streak ever recorded in sessions. */
  const bestStreak = computed(() => {
    if (sessions.value.length === 0) return 0

    const uniqueDates = [...new Set(sessions.value.map((s) => s.date))]
      .sort()
      .map((iso) => isoToDate(iso))

    let best = 1
    let current = 1

    for (let i = 1; i < uniqueDates.length; i++) {
      const prev = uniqueDates[i - 1]
      const curr = uniqueDates[i]
      const diff = Math.round(
        (curr.getTime() - prev.getTime()) / (1000 * 60 * 60 * 24),
      )
      if (diff === 1) {
        current++
        if (current > best) best = current
      } else {
        current = 1
      }
    }

    return best
  })

  /** Activity data for the current week (Mon to Sun). */
  const weeklyActivity = computed<DayActivity[]>(() => {
    const dates = currentWeekDates()
    return dates.map((iso, idx) => ({
      date: iso,
      totalSeconds: sessionsByDate.value.get(iso) ?? 0,
      label: DAY_LABELS[idx],
    }))
  })

  /** Per-book total time, sorted by most read. */
  const bookActivity = computed<BookActivity[]>(() => {
    const map = new Map<
      string,
      { bookTitle: string; bookAuthor: string | null; totalSeconds: number }
    >()
    for (const s of sessions.value) {
      const existing = map.get(s.bookId)
      if (existing) {
        existing.totalSeconds += s.durationSeconds
      } else {
        map.set(s.bookId, {
          bookTitle: s.bookTitle,
          bookAuthor: s.bookAuthor,
          totalSeconds: s.durationSeconds,
        })
      }
    }
    return [...map.entries()]
      .map(([bookId, v]) => ({ bookId, ...v }))
      .sort((a, b) => b.totalSeconds - a.totalSeconds)
  })

  /**
   * Gapless all-time activity series.
   * Buckets by day when <=90 days of history, by week otherwise.
   * Always ends with today so the right edge is anchored.
   */
  const allTimeDaily = computed<ChartPoint[]>(() => {
    const map = sessionsByDate.value
    if (map.size === 0) return []

    const allDates = [...map.keys()].sort()
    const firstDate = isoToDate(allDates[0])
    const today = new Date()
    today.setHours(0, 0, 0, 0)

    const totalDays = Math.round(
      (today.getTime() - firstDate.getTime()) / (1000 * 60 * 60 * 24),
    ) + 1

    const todayStr = todayISO()

    if (totalDays <= 90) {
      // Daily buckets
      const points: ChartPoint[] = []
      const cursor = new Date(firstDate)
      while (cursor <= today) {
        const iso = dateToISO(cursor)
        points.push({
          date: iso,
          seconds: map.get(iso) ?? 0,
          isToday: iso === todayStr,
        })
        cursor.setDate(cursor.getDate() + 1)
      }
      return points
    } else {
      // Weekly buckets - group Mon to Sun
      const points: ChartPoint[] = []
      // Snap firstDate back to its Monday
      const dow = (firstDate.getDay() + 6) % 7
      const weekStart = new Date(firstDate)
      weekStart.setDate(firstDate.getDate() - dow)
      weekStart.setHours(0, 0, 0, 0)

      const cursor = new Date(weekStart)
      while (cursor <= today) {
        const weekEnd = new Date(cursor)
        weekEnd.setDate(cursor.getDate() + 6)

        let totalSec = 0
        let containsToday = false
        for (let d = new Date(cursor); d <= weekEnd; d.setDate(d.getDate() + 1)) {
          const iso = dateToISO(d)
          totalSec += map.get(iso) ?? 0
          if (iso === todayStr) containsToday = true
        }

        points.push({
          date: dateToISO(cursor),
          seconds: totalSec,
          isToday: containsToday,
        })
        cursor.setDate(cursor.getDate() + 7)
      }
      return points
    }
  })

  // ── Extended analytics & helpers ──────────────────────────────────────────

  const todaySeconds = computed(() => sessionsByDate.value.get(todayISO()) ?? 0)

  const thisWeekSeconds = computed(() =>
    weeklyActivity.value.reduce((acc, d) => acc + d.totalSeconds, 0),
  )

  const completionRate = computed(() => {
    if (totalBooks.value === 0) return 0
    return Math.round((booksFinished.value / totalBooks.value) * 100)
  })

  const activeDaysCount = computed(() => sessionsByDate.value.size)

  const dailyAverageSeconds = computed(() => {
    if (sessionsByDate.value.size === 0) return 0
    return Math.round(totalReadingSeconds.value / sessionsByDate.value.size)
  })

  const peakDayWeekly = computed(() => {
    if (weeklyActivity.value.length === 0) return null
    let max = weeklyActivity.value[0]
    for (const d of weeklyActivity.value) {
      if (d.totalSeconds > max.totalSeconds) {
        max = d
      }
    }
    return max.totalSeconds > 0 ? max : null
  })

  const epubCount = computed(
    () => entries.value.filter((e) => e.book.format === 'epub').length,
  )

  const pdfCount = computed(
    () => entries.value.filter((e) => e.book.format === 'pdf').length,
  )

  const streakDaysThisWeek = computed(() => {
    const todayStr = todayISO()
    return weeklyActivity.value.map((d) => ({
      ...d,
      isToday: d.date === todayStr,
      active: d.totalSeconds > 0,
    }))
  })

  const timeOfDayDistribution = computed(() => {
    let morning = 0
    let afternoon = 0
    let evening = 0
    let night = 0
    let mCount = 0
    let aCount = 0
    let eCount = 0
    let nCount = 0

    for (const s of sessionsState.sessions) {
      const h = new Date(s.startedAt).getHours()
      if (h >= 5 && h < 12) {
        morning += s.durationSeconds
        mCount++
      } else if (h >= 12 && h < 17) {
        afternoon += s.durationSeconds
        aCount++
      } else if (h >= 17 && h < 22) {
        evening += s.durationSeconds
        eCount++
      } else {
        night += s.durationSeconds
        nCount++
      }
    }

    const total = morning + afternoon + evening + night
    const calcPct = (sec: number) => (total > 0 ? Math.round((sec / total) * 100) : 0)

    return [
      {
        id: 'morning',
        label: 'Morning',
        period: '5:00 AM - 12:00 PM',
        icon: 'wb_sunny',
        seconds: morning,
        percentage: calcPct(morning),
        sessionCount: mCount,
      },
      {
        id: 'afternoon',
        label: 'Afternoon',
        period: '12:00 PM - 5:00 PM',
        icon: 'light_mode',
        seconds: afternoon,
        percentage: calcPct(afternoon),
        sessionCount: aCount,
      },
      {
        id: 'evening',
        label: 'Evening',
        period: '5:00 PM - 10:00 PM',
        icon: 'dark_mode',
        seconds: evening,
        percentage: calcPct(evening),
        sessionCount: eCount,
      },
      {
        id: 'night',
        label: 'Night',
        period: '10:00 PM - 5:00 AM',
        icon: 'bedtime',
        seconds: night,
        percentage: calcPct(night),
        sessionCount: nCount,
      },
    ]
  })

  const enrichedBookActivity = computed(() => {
    const totalSec = totalReadingSeconds.value
    const secMap = new Map<string, number>()

    for (const s of sessions.value) {
      secMap.set(s.bookId, (secMap.get(s.bookId) ?? 0) + s.durationSeconds)
    }

    return entries.value
      .map((entry) => {
        const sec = secMap.get(entry.book.id) ?? 0
        const pct = totalSec > 0 ? Math.round((sec / totalSec) * 100) : 0
        return {
          bookId: entry.book.id,
          book: entry.book,
          bookTitle: entry.book.title,
          bookAuthor: entry.book.author,
          totalSeconds: sec,
          progress: entry.progress,
          format: entry.book.format,
          cover: entry.book.cover,
          lastRead: entry.lastRead,
          percentageOfTotal: pct,
          rawEntry: entry,
        }
      })
      .filter((b) => b.totalSeconds > 0 || b.progress > 0)
      .sort((a, b) => b.totalSeconds - a.totalSeconds || (b.lastRead ?? 0) - (a.lastRead ?? 0))
  })

  function getTrendPoints(timeframe: '7d' | '30d' | '90d' | 'all'): ChartPoint[] {
    if (timeframe === 'all') {
      return allTimeDaily.value
    }

    const days = timeframe === '7d' ? 7 : timeframe === '30d' ? 30 : 90
    const today = new Date()
    today.setHours(0, 0, 0, 0)
    const map = sessionsByDate.value
    const todayStr = todayISO()
    const points: ChartPoint[] = []

    for (let i = days - 1; i >= 0; i--) {
      const d = new Date(today)
      d.setDate(today.getDate() - i)
      const iso = dateToISO(d)
      points.push({
        date: iso,
        seconds: map.get(iso) ?? 0,
        isToday: iso === todayStr,
      })
    }

    return points
  }

  // ── Formatting helpers ─────────────────────────────────────────────────────

  function formatDuration(seconds: number): string {
    if (seconds === 0) return '0 min'
    const h = Math.floor(seconds / 3600)
    const m = Math.floor((seconds % 3600) / 60)
    if (h > 0 && m > 0) return `${h}h ${m}m`
    if (h > 0) return `${h}h`
    return `${m} min`
  }

  function formatDurationCompact(seconds: number): string {
    if (seconds === 0) return '0m'
    const h = Math.floor(seconds / 3600)
    const m = Math.floor((seconds % 3600) / 60)
    if (h > 0 && m > 0) return `${h}h ${m}m`
    if (h > 0) return `${h}h`
    return `${m}m`
  }

  return {
    // Library
    totalBooks,
    booksFinished,
    booksInProgress,
    booksUnstarted,
    averageProgress,
    lastReadEntry,
    epubCount,
    pdfCount,
    completionRate,
    // Sessions
    totalReadingSeconds,
    todaySeconds,
    thisWeekSeconds,
    dailyAverageSeconds,
    activeDaysCount,
    sessionHistory,
    currentStreak,
    bestStreak,
    peakDayWeekly,
    streakDaysThisWeek,
    weeklyActivity,
    bookActivity,
    enrichedBookActivity,
    allTimeDaily,
    timeOfDayDistribution,
    // Methods
    getTrendPoints,
    // Helpers
    formatDuration,
    formatDurationCompact,
  }
}
