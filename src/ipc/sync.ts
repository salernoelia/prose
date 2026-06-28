/**
 * Sync IPC wrappers: the only place `invoke` appears for sync commands.
 *
 * All commands map to `src-tauri/src/ipc/sync.rs`. Network I/O runs on a
 * blocking thread in the Rust core; these functions just marshal the calls.
 */
import { invoke } from '@tauri-apps/api/core'
import type { RemoteBookDto, SyncStatusDto } from './types'

/** Configure the WebDAV server. Validates connectivity before saving. */
export function syncConfigure(url: string, username: string, password: string): Promise<void> {
  return invoke('sync_configure', { url, username, password })
}

/** Return the current sync configuration (URL and username only, no password). */
export function syncStatus(): Promise<SyncStatusDto> {
  return invoke('sync_status')
}

/** Remove stored credentials and clear the sync configuration. */
export function syncDisconnect(): Promise<void> {
  return invoke('sync_disconnect')
}

/**
 * Trigger a full sync in the background. Returns immediately; progress is
 * reported via `sync:progress` events and completion via `sync:finished`.
 */
export function syncTrigger(): Promise<void> {
  return invoke('sync_trigger')
}

/** List book files available on the remote server. */
export function syncListRemote(): Promise<RemoteBookDto[]> {
  return invoke('sync_list_remote')
}

/** Download a book from the given remote path and import it into the library. */
export function syncDownloadBook(path: string): Promise<void> {
  return invoke('sync_download_book', { path })
}
