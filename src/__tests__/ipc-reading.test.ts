import { describe, it, expect, vi, beforeEach } from 'vitest'
import { readingSavePosition, readingGetPosition } from '../ipc/reading'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'
const mockedInvoke = vi.mocked(invoke)

describe('reading IPC wrappers', () => {
  beforeEach(() => {
    mockedInvoke.mockReset()
  })

  it('readingSavePosition invokes the correct command with bookId and locator', async () => {
    const locator = { payload: 'epubcfi(/6/4)', progression: 0.25 }
    const returned = { locator, updatedAt: 1_000 }
    mockedInvoke.mockResolvedValue(returned)

    const result = await readingSavePosition('abc123', locator)

    expect(mockedInvoke).toHaveBeenCalledWith('reading_save_position', {
      bookId: 'abc123',
      locator,
    })
    expect(result).toEqual(returned)
  })

  it('readingGetPosition returns null when the book has no saved position', async () => {
    mockedInvoke.mockResolvedValue(null)

    const result = await readingGetPosition('abc123')

    expect(mockedInvoke).toHaveBeenCalledWith('reading_get_position', { bookId: 'abc123' })
    expect(result).toBeNull()
  })

  it('readingGetPosition returns a ProgressDto when a position exists', async () => {
    const saved = {
      locator: { payload: 'epubcfi(/6/8)', progression: 0.5 },
      updatedAt: 2_000,
    }
    mockedInvoke.mockResolvedValue(saved)

    const result = await readingGetPosition('abc123')

    expect(result).toEqual(saved)
  })
})
