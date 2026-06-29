import { describe, it, expect, vi, beforeEach } from 'vitest'
import { initLibraryStore, libraryState, updateLibraryQuery } from '../stores/library'
import { useLibrary } from '../composables/useLibrary'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}))

import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
const mockedInvoke = vi.mocked(invoke)
const mockedListen = vi.mocked(listen)

describe('Library Store and Composable', () => {
  beforeEach(() => {
    mockedInvoke.mockReset()
    mockedListen.mockReset()
  })

  it('initLibraryStore loads catalog and registers listeners', async () => {
    const mockEntries = [
      {
        book: {
          id: '1',
          format: 'epub' as const,
          title: 'Alice',
          author: 'Lewis',
          cover: null,
        },
        progress: 0.1,
        lastRead: 100,
      },
    ]
    mockedInvoke.mockResolvedValue(mockEntries)

    await initLibraryStore()

    expect(mockedInvoke).toHaveBeenCalledWith('library_list', {
      query: { search: null, sort: 'progress', descending: true },
    })
    expect(libraryState.entries).toEqual(mockEntries)
    expect(libraryState.loaded).toBe(true)
  })

  it('updateLibraryQuery patches query and reloads library', async () => {
    mockedInvoke.mockResolvedValue([])

    await updateLibraryQuery({ search: 'Alice' })

    expect(mockedInvoke).toHaveBeenCalledWith('library_list', {
      query: { search: 'Alice', sort: 'progress', descending: true },
    })
    expect(libraryState.query.search).toBe('Alice')
  })

  it('useLibrary composable provides computed properties', () => {
    const { entries, query } = useLibrary()

    expect(entries.value).toEqual([])
    expect(query.value.search).toBe('Alice')
  })
})
