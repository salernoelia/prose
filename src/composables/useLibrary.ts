import { computed } from 'vue'
import {
  libraryState,
  initLibraryStore,
  updateLibraryQuery,
  importBook,
  removeBook,
} from '../stores/library'

export function useLibrary() {
  initLibraryStore()

  const entries = computed(() => libraryState.entries)
  const query = computed(() => libraryState.query)
  const loaded = computed(() => libraryState.loaded)
  const importing = computed(() => libraryState.importing)
  const importMessage = computed(() => libraryState.importMessage)
  const importFraction = computed(() => libraryState.importFraction)

  return {
    entries,
    query,
    loaded,
    importing,
    importMessage,
    importFraction,
    updateLibraryQuery,
    importBook,
    removeBook,
  }
}
