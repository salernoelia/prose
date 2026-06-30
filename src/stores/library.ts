import { reactive, readonly } from 'vue'
import type { LibraryEntryDto, LibraryQueryDto, BookDto } from '../ipc/types'
import { libraryList, libraryImportBook, libraryRemove } from '../ipc/library'
import { onLibraryChanged, onImportProgress } from '../ipc/events'
import { syncState, triggerSync } from './sync'

const defaultQuery: LibraryQueryDto = {
  search: null,
  sort: 'progress',
  descending: true,
}

// Sort key and direction persist locally so the library opens the way the user
// left it. Search stays transient. This is a device preference, not synced.
const SORT_PREFS_KEY = 'prose.library.sort'

function loadSortPrefs(): Pick<LibraryQueryDto, 'sort' | 'descending'> {
  try {
    const raw = localStorage.getItem(SORT_PREFS_KEY)
    if (raw) {
      const parsed = JSON.parse(raw) as Partial<LibraryQueryDto>
      return {
        sort: parsed.sort ?? defaultQuery.sort,
        descending: parsed.descending ?? defaultQuery.descending,
      }
    }
  } catch (err) {
    console.error('Failed to load library sort preferences:', err)
  }
  return { sort: defaultQuery.sort, descending: defaultQuery.descending }
}

function saveSortPrefs(query: LibraryQueryDto): void {
  try {
    localStorage.setItem(
      SORT_PREFS_KEY,
      JSON.stringify({ sort: query.sort, descending: query.descending })
    )
  } catch (err) {
    console.error('Failed to save library sort preferences:', err)
  }
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
  query: { ...defaultQuery, ...loadSortPrefs() },
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
  if ('sort' in patch || 'descending' in patch) {
    saveSortPrefs(state.query)
  }
  await reloadLibrary()
}

export async function importBook(filePath: string): Promise<BookDto> {
  state.importing = true
  state.importMessage = 'Starting import...'
  state.importFraction = 0.0
  try {
    const book = await libraryImportBook(filePath)
    state.importing = false
    if (syncState.configured) {
      void triggerSync()
    }
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
    if (syncState.configured) {
      void triggerSync()
    }
  } catch (err) {
    console.error('Failed to remove book:', err)
    throw err
  }
}
