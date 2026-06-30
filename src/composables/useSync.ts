import { computed } from 'vue'
import {
  syncState,
  initSyncStore,
  triggerSync,
  refreshSyncConfig,
  dismissSyncResult,
} from '../stores/sync'

export function useSync() {
  initSyncStore()

  const configured = computed(() => syncState.configured)
  const syncing = computed(() => syncState.syncing)
  const progressMessage = computed(() => syncState.progressMessage)
  const progressFraction = computed(() => syncState.progressFraction)
  const lastFinishedResult = computed(() => syncState.lastFinishedResult)
  const hasSyncError = computed(() => syncState.hasSyncError)

  return {
    configured,
    syncing,
    progressMessage,
    progressFraction,
    lastFinishedResult,
    hasSyncError,
    triggerSync,
    refreshSyncConfig,
    dismissSyncResult,
  }
}
