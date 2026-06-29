import { describe, it, expect, vi, beforeEach } from 'vitest'
import {
  annotationAddBookmark,
  annotationListBookmarks,
  annotationDeleteBookmark,
  annotationAddHighlight,
  annotationListHighlights,
  annotationDeleteHighlight,
} from '../ipc/annotation'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'
const mockedInvoke = vi.mocked(invoke)

describe('annotation IPC wrappers', () => {
  beforeEach(() => {
    mockedInvoke.mockReset()
  })

  it('annotationAddBookmark passes bookId and locator', async () => {
    const locator = { payload: 'epubcfi(/6/4)', progression: 0.25 }
    const created = { id: 'bm_1', bookId: 'abc', locator, createdAt: 10 }
    mockedInvoke.mockResolvedValue(created)

    const result = await annotationAddBookmark('abc', locator)

    expect(mockedInvoke).toHaveBeenCalledWith('annotation_add_bookmark', {
      bookId: 'abc',
      locator,
    })
    expect(result).toEqual(created)
  })

  it('annotationListBookmarks passes bookId', async () => {
    mockedInvoke.mockResolvedValue([])
    await annotationListBookmarks('abc')
    expect(mockedInvoke).toHaveBeenCalledWith('annotation_list_bookmarks', { bookId: 'abc' })
  })

  it('annotationDeleteBookmark passes bookmarkId', async () => {
    mockedInvoke.mockResolvedValue(undefined)
    await annotationDeleteBookmark('bm_1')
    expect(mockedInvoke).toHaveBeenCalledWith('annotation_delete_bookmark', { bookmarkId: 'bm_1' })
  })

  it('annotationAddHighlight passes bookId, locator, text, and color', async () => {
    const locator = { payload: 'epubcfi(/6/4,/2,/8)', progression: 0.3 }
    const created = {
      id: 'hl_1',
      bookId: 'abc',
      locator,
      text: 'a passage',
      color: 'yellow',
      createdAt: 20,
    }
    mockedInvoke.mockResolvedValue(created)

    const result = await annotationAddHighlight('abc', locator, 'a passage', 'yellow')

    expect(mockedInvoke).toHaveBeenCalledWith('annotation_add_highlight', {
      bookId: 'abc',
      locator,
      text: 'a passage',
      color: 'yellow',
    })
    expect(result).toEqual(created)
  })

  it('annotationListHighlights passes bookId', async () => {
    mockedInvoke.mockResolvedValue([])
    await annotationListHighlights('abc')
    expect(mockedInvoke).toHaveBeenCalledWith('annotation_list_highlights', { bookId: 'abc' })
  })

  it('annotationDeleteHighlight passes highlightId', async () => {
    mockedInvoke.mockResolvedValue(undefined)
    await annotationDeleteHighlight('hl_1')
    expect(mockedInvoke).toHaveBeenCalledWith('annotation_delete_highlight', { highlightId: 'hl_1' })
  })
})
