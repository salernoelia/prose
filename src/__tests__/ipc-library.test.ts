import { describe, it, expect, vi, beforeEach } from 'vitest'
import { libraryImportBook, libraryList, libraryRemove } from '../ipc/library'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'
const mockedInvoke = vi.mocked(invoke)

describe('library IPC wrappers', () => {
  beforeEach(() => {
    mockedInvoke.mockReset()
  })

  it('libraryImportBook invokes the correct command', async () => {
    const fakeBook = {
      id: 'book123',
      format: 'epub' as const,
      title: 'Mock Title',
      author: 'Mock Author',
      cover: 'covers/book123.png',
    }
    mockedInvoke.mockResolvedValue(fakeBook)

    const result = await libraryImportBook('/path/to/book.epub')

    expect(mockedInvoke).toHaveBeenCalledWith('library_import_book', { path: '/path/to/book.epub' })
    expect(result).toEqual(fakeBook)
  })

  it('libraryList invokes the correct command', async () => {
    const fakeEntries = [
      {
        book: {
          id: 'book123',
          format: 'epub' as const,
          title: 'Mock Title',
          author: 'Mock Author',
          cover: 'covers/book123.png',
        },
        progress: 0.5,
        lastRead: 1625097600000,
      },
    ]
    mockedInvoke.mockResolvedValue(fakeEntries)

    const query = { search: 'Mock', sort: 'title' as const, descending: false }
    const result = await libraryList(query)

    expect(mockedInvoke).toHaveBeenCalledWith('library_list', { query })
    expect(result).toEqual(fakeEntries)
  })

  it('libraryRemove invokes the correct command', async () => {
    mockedInvoke.mockResolvedValue(undefined)

    await libraryRemove('book123')

    expect(mockedInvoke).toHaveBeenCalledWith('library_remove', { id: 'book123' })
  })
})
