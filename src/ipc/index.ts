// The IPC boundary: the only place `invoke` and `listen` appear. Typed command
// wrappers and event listeners are re-exported here as each feature lands.
export { getSettings, patchSettings } from './settings'
export { libraryImportBook, libraryList, libraryRemove } from './library'
export {
  readingSavePosition,
  readingGetPosition,
  readingLogSession,
  readingListSessions,
} from './reading'
export {
  annotationAddBookmark,
  annotationListBookmarks,
  annotationDeleteBookmark,
  annotationAddHighlight,
  annotationListHighlights,
  annotationDeleteHighlight,
} from './annotation'
export { dictionaryLookup } from './dictionary'
export {
  syncConfigure,
  syncStatus,
  syncDisconnect,
  syncTrigger,
  syncListRemote,
  syncDownloadBook,
} from './sync'
export {
  onSettingsChanged,
  onLibraryChanged,
  onImportProgress,
  onSyncProgress,
  onSyncFinished,
} from './events'
export type {
  SettingsDto,
  SettingsPatchDto,
  SettingsChangedPayload,
  Theme,
  Format,
  BookDto,
  LibraryEntryDto,
  SortKey,
  LibraryQueryDto,
  ImportProgressPayload,
  LocatorDto,
  ProgressDto,
  ReadingSessionDto,
  BookmarkDto,
  HighlightDto,
  DefinitionDto,
  SyncStatusDto,
  RemoteBookDto,
  SyncProgressPayload,
  SyncFinishedPayload,
} from './types'
