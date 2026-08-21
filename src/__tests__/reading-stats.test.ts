import { describe, it, expect, beforeEach, vi } from 'vitest'

vi.mock('../ipc/library', () => ({
  libraryList: vi.fn(),
  libraryImportBook: vi.fn(),
  libraryRemove: vi.fn(),
  librarySetArchived: vi.fn(),
}))

vi.mock('../ipc/reading', () => ({
  readingListSessions: vi.fn(),
  readingLogSession: vi.fn(),
  readingDeleteSession: vi.fn(),
}))

vi.mock('../ipc/events', () => ({
  onLibraryChanged: vi.fn().mockResolvedValue(() => {}),
  onImportProgress: vi.fn().mockResolvedValue(() => {}),
  onSyncFinished: vi.fn().mockResolvedValue(() => {}),
}))

import { libraryList } from '../ipc/library'
import { readingListSessions } from '../ipc/reading'
import { reloadLibrary } from '../stores/library'
import { refreshSessions } from '../stores/sessions'
import { useReadingStats } from '../composables/useReadingStats'

const mockLibraryList = vi.mocked(libraryList)
const mockReadingListSessions = vi.mocked(readingListSessions)

describe('useReadingStats', () => {
  beforeEach(async () => {
    mockLibraryList.mockReset()
    mockReadingListSessions.mockReset()
  })

  it('formats duration correctly', () => {
    const { formatDuration } = useReadingStats()
    expect(formatDuration(0)).toBe('0 min')
    expect(formatDuration(45)).toBe('0 min')
    expect(formatDuration(120)).toBe('2 min')
    expect(formatDuration(3600)).toBe('1h')
    expect(formatDuration(3660)).toBe('1h 1m')
  })

  it('computes library totals correctly', async () => {
    const entries = [
      {
        book: { id: '1', title: 'Book 1', author: 'Author A', format: 'epub' as const, file_path: '', cover: null, created_at: 0 },
        progress: 0,
        lastRead: null,
        archived: false,
      },
      {
        book: { id: '2', title: 'Book 2', author: 'Author B', format: 'pdf' as const, file_path: '', cover: null, created_at: 0 },
        progress: 0.5,
        lastRead: 100,
        archived: false,
      },
      {
        book: { id: '3', title: 'Book 3', author: 'Author C', format: 'epub' as const, file_path: '', cover: null, created_at: 0 },
        progress: 1.0,
        lastRead: 200,
        archived: false,
      },
    ]

    mockLibraryList.mockResolvedValue(entries)
    mockReadingListSessions.mockResolvedValue([])

    await reloadLibrary()
    await refreshSessions()

    const stats = useReadingStats()
    expect(stats.totalBooks.value).toBe(3)
    expect(stats.booksUnstarted.value).toBe(1)
    expect(stats.booksInProgress.value).toBe(1)
    expect(stats.booksFinished.value).toBe(1)
    expect(stats.averageProgress.value).toBe(0.5)
  })

  it('computes reading sessions and streaks', async () => {
    const now = Date.now()
    const sessions = [
      {
        id: 's1',
        bookId: '1',
        startedAt: now - 3600 * 1000,
        durationSeconds: 1800,
      },
    ]

    const entries = [
      {
        book: { id: '1', title: 'Book 1', author: 'Author A', format: 'epub' as const, file_path: '', cover: null, created_at: 0 },
        progress: 0.2,
        lastRead: now,
        archived: false,
      },
    ]

    mockLibraryList.mockResolvedValue(entries)
    mockReadingListSessions.mockResolvedValue(sessions)

    await reloadLibrary()
    await refreshSessions()

    const stats = useReadingStats()
    expect(stats.totalReadingSeconds.value).toBe(1800)
    expect(stats.sessionHistory.value.length).toBe(1)
    expect(stats.sessionHistory.value[0].bookTitle).toBe('Book 1')
  })

  it('computes extended analytics and distributions correctly', async () => {
    const morningTime = new Date('2026-08-20T08:30:00').getTime()
    const eveningTime = new Date('2026-08-20T19:00:00').getTime()

    const sessions = [
      { id: 's1', bookId: '1', startedAt: morningTime, durationSeconds: 1200 },
      { id: 's2', bookId: '1', startedAt: eveningTime, durationSeconds: 2400 },
    ]

    const entries = [
      {
        book: { id: '1', title: 'Book 1', author: 'Author A', format: 'epub' as const, file_path: '', cover: null, created_at: 0 },
        progress: 0.8,
        lastRead: eveningTime,
        archived: false,
      },
      {
        book: { id: '2', title: 'Book 2', author: 'Author B', format: 'pdf' as const, file_path: '', cover: null, created_at: 0 },
        progress: 1.0,
        lastRead: null,
        archived: false,
      },
    ]

    mockLibraryList.mockResolvedValue(entries)
    mockReadingListSessions.mockResolvedValue(sessions)

    await reloadLibrary()
    await refreshSessions()

    const stats = useReadingStats()
    expect(stats.totalReadingSeconds.value).toBe(3600)
    expect(stats.completionRate.value).toBe(50)
    expect(stats.epubCount.value).toBe(1)
    expect(stats.pdfCount.value).toBe(1)
    expect(stats.formatDurationCompact(3600)).toBe('1h')
    expect(stats.formatDurationCompact(3660)).toBe('1h 1m')
    expect(stats.formatDurationCompact(120)).toBe('2m')

    const dist = stats.timeOfDayDistribution.value
    const morning = dist.find((d) => d.id === 'morning')
    const evening = dist.find((d) => d.id === 'evening')
    expect(morning?.seconds).toBe(1200)
    expect(evening?.seconds).toBe(2400)

    const trends7d = stats.getTrendPoints('7d')
    expect(trends7d.length).toBe(7)

    const enriched = stats.enrichedBookActivity.value
    expect(enriched.length).toBe(2)
    expect(enriched[0].bookTitle).toBe('Book 1')
    expect(enriched[0].totalSeconds).toBe(3600)
  })
})

