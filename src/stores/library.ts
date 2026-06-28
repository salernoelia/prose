import { reactive, readonly } from 'vue'
import type { LibraryEntryDto, LibraryQueryDto, BookDto } from '../ipc/types'
import { libraryList, libraryImportBook, libraryRemove } from '../ipc/library'
import { onLibraryChanged, onImportProgress } from '../ipc/events'

const defaultQuery: LibraryQueryDto = {
  search: null,
  sort: 'title',
  descending: false,
}

const state = reactive<{
  entries: LibraryEntryDto[]
  query: LibraryQueryDto
  loaded: boolean
  importing: boolean
  importMessage: string
  importFraction: number
}>({
  entries: [],
  query: { ...defaultQuery },
  loaded: false,
  importing: false,
  importMessage: '',
  importFraction: 0,
})

let initPromise: Promise<void> | null = null

export function initLibraryStore(): Promise<void> {
  if (initPromise) return initPromise

  initPromise = (async () => {
    try {
      await reloadLibrary()
      state.loaded = true
    } catch (err) {
      console.error('Failed to load library:', err)
    }

    try {
      await onLibraryChanged(async () => {
        await reloadLibrary()
      })
    } catch (err) {
      console.error('Failed to listen to library:changed:', err)
    }

    try {
      await onImportProgress((payload) => {
        state.importing = payload.fraction < 1.0
        state.importMessage = payload.message
        state.importFraction = payload.fraction
      })
    } catch (err) {
      console.error('Failed to listen to import:progress:', err)
    }
  })()

  return initPromise
}

export async function reloadLibrary(): Promise<void> {
  try {
    const data = await libraryList(state.query)
    state.entries = data
  } catch (err) {
    console.error('Failed to reload library:', err)
  }
}

export const libraryState = readonly(state)

export async function updateLibraryQuery(patch: Partial<LibraryQueryDto>): Promise<void> {
  Object.assign(state.query, patch)
  await reloadLibrary()
}

export async function importBook(filePath: string): Promise<BookDto> {
  state.importing = true
  state.importMessage = 'Starting import...'
  state.importFraction = 0.0
  try {
    const book = await libraryImportBook(filePath)
    state.importing = false
    return book
  } catch (err) {
    state.importing = false
    console.error('Failed to import book:', err)
    throw err
  }
}

export async function removeBook(id: string): Promise<void> {
  try {
    await libraryRemove(id)
  } catch (err) {
    console.error('Failed to remove book:', err)
    throw err
  }
}
