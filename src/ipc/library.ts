/**
 * Library command wrappers: the only place `invoke` appears for library.
 *
 * Each function mirrors a Rust `#[tauri::command]` in
 * `src-tauri/src/ipc/library.rs`.
 */
import { invoke } from '@tauri-apps/api/core'
import type { BookDto, LibraryEntryDto, LibraryQueryDto } from './types'

/** Import a book file by path. */
export function libraryImportBook(path: string): Promise<BookDto> {
  return invoke<BookDto>('library_import_book', { path })
}

/** List library entries matching the query. */
export function libraryList(query: LibraryQueryDto): Promise<LibraryEntryDto[]> {
  return invoke<LibraryEntryDto[]>('library_list', { query })
}

/** Remove a book and its associated data/files from the library. */
export function libraryRemove(id: string): Promise<void> {
  return invoke<void>('library_remove', { id })
}

/** Archive or unarchive a book, toggling its visibility in the default view. */
export function librarySetArchived(id: string, archived: boolean): Promise<void> {
  return invoke<void>('library_set_archived', { id, archived })
}
