/**
 * Annotation IPC wrappers: the only place `invoke` appears for bookmarks and
 * highlights (FR-NOTE-01/02).
 *
 * The location travels as an opaque `LocatorDto` the Rust core never interprets;
 * highlights also carry the selected `text` so they survive re-pagination. Ids
 * are minted in the domain, so the returned DTO carries the stable id.
 */
import { invoke } from '@tauri-apps/api/core'
import type { BookmarkDto, HighlightDto, LocatorDto } from './types'

/** Bookmark the given location. Returns the created bookmark. */
export function annotationAddBookmark(bookId: string, locator: LocatorDto): Promise<BookmarkDto> {
  return invoke('annotation_add_bookmark', { bookId, locator })
}

/** List the bookmarks for a book, in store order. */
export function annotationListBookmarks(bookId: string): Promise<BookmarkDto[]> {
  return invoke('annotation_list_bookmarks', { bookId })
}

/** Delete a bookmark by its id. */
export function annotationDeleteBookmark(bookmarkId: string): Promise<void> {
  return invoke('annotation_delete_bookmark', { bookmarkId })
}

/** Persist a highlight over a selected text range. Returns the created highlight. */
export function annotationAddHighlight(
  bookId: string,
  locator: LocatorDto,
  text: string,
  color: string | null,
): Promise<HighlightDto> {
  return invoke('annotation_add_highlight', { bookId, locator, text, color })
}

/** List the highlights for a book, in store order. */
export function annotationListHighlights(bookId: string): Promise<HighlightDto[]> {
  return invoke('annotation_list_highlights', { bookId })
}

/** Delete a highlight by its id. */
export function annotationDeleteHighlight(highlightId: string): Promise<void> {
  return invoke('annotation_delete_highlight', { highlightId })
}
