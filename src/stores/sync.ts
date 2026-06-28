import { reactive, readonly } from 'vue'
import { syncStatus, syncTrigger } from '../ipc/sync'
import { onSyncProgress, onSyncFinished } from '../ipc/events'
import { reloadLibrary } from './library'
import { reloadSettings } from './settings'

const state = reactive<{
  configured: boolean
  syncing: boolean
  progressMessage: string
  progressFraction: number
  lastFinishedResult: { success: boolean; message: string } | null
}>({
  configured: false,
  syncing: false,
  progressMessage: '',
  progressFraction: 0,
  lastFinishedResult: null,
})

let initPromise: Promise<void> | null = null

export function initSyncStore(): Promise<void> {
  if (initPromise) return initPromise

  initPromise = (async () => {
    try {
      const status = await syncStatus()
      state.configured = status.configured
      
      // Auto-sync on startup if configured
      if (status.configured) {
        void triggerSync()
      }
    } catch (err) {
      console.error('Failed to get sync status:', err)
    }

    try {
      await onSyncProgress((payload) => {
        state.syncing = true
        state.progressMessage = formatProgressStage(payload.stage)
        state.progressFraction = payload.fraction
      })
    } catch (err) {
      console.error('Failed to listen to sync:progress:', err)
    }

    try {
      await onSyncFinished(async (payload) => {
        state.syncing = false
        state.lastFinishedResult = {
          success: payload.success,
          message: payload.message,
        }
        if (payload.success) {
          // Sync succeeded - reload library and settings to reflect remote changes
          await reloadLibrary()
          await reloadSettings()
        }
      })
    } catch (err) {
      console.error('Failed to listen to sync:finished:', err)
    }

    // Periodically sync every 5 minutes if configured and not already syncing
    setInterval(() => {
      if (state.configured && !state.syncing) {
        void triggerSync()
      }
    }, 5 * 60 * 1000)
  })()

  return initPromise
}

export async function refreshSyncConfig(): Promise<void> {
  try {
    const status = await syncStatus()
    state.configured = status.configured
  } catch (err) {
    console.error('Failed to refresh sync status:', err)
  }
}

export async function triggerSync(): Promise<void> {
  if (state.syncing) return
  state.syncing = true
  state.progressMessage = 'Connecting...'
  state.progressFraction = 0
  state.lastFinishedResult = null
  try {
    await syncTrigger()
  } catch (err) {
    state.syncing = false
    const msg = err && typeof err === 'object' && 'message' in err ? String((err as { message: string }).message) : String(err)
    state.lastFinishedResult = {
      success: false,
      message: msg,
    }
    console.error('Failed to trigger sync:', err)
  }
}

export function dismissSyncResult(): void {
  state.lastFinishedResult = null
}

function formatProgressStage(stage: string): string {
  switch (stage) {
    case 'syncing_settings':
      return 'Syncing settings...'
    case 'syncing_progress':
      return 'Syncing reading progress...'
    case 'syncing_bookmarks':
      return 'Syncing bookmarks...'
    case 'syncing_highlights':
      return 'Syncing highlights...'
    case 'syncing_books':
      return 'Syncing book files...'
    case 'uploading_book':
      return 'Uploading books...'
    case 'done':
      return 'Sync complete'
    default:
      return 'Syncing...'
  }
}

export const syncState = readonly(state)

export function resetSyncStoreForTesting(): void {
  state.configured = false
  state.syncing = false
  state.progressMessage = ''
  state.progressFraction = 0
  state.lastFinishedResult = null
}
