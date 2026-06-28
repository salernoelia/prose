// Reactive state, seeded from Rust on startup and updated by IPC events. Stores
// hold a display copy only; the authority is always the Rust core.
export { settingsState, initSettingsStore, updateSettings } from './settings'
export {
  libraryState,
  initLibraryStore,
  reloadLibrary,
  updateLibraryQuery,
  importBook,
  removeBook,
} from './library'
export {}
