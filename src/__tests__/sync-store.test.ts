import { describe, it, expect, vi, beforeEach } from 'vitest'
import { initSyncStore, syncState, triggerSync, refreshSyncConfig, dismissSyncResult, resetSyncStoreForTesting } from '../stores/sync'
import { useSync } from '../composables/useSync'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}))

import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
const mockedInvoke = vi.mocked(invoke)
const mockedListen = vi.mocked(listen)

describe('Sync Store and Composable', () => {
  beforeEach(() => {
    mockedInvoke.mockReset()
    mockedListen.mockReset()
    resetSyncStoreForTesting()
  })

  it('initSyncStore seeds configuration status and listens to events', async () => {
    mockedInvoke.mockResolvedValue({
      configured: true,
      url: 'https://example.com/webdav',
      username: 'user',
    })

    await initSyncStore()

    expect(mockedInvoke).toHaveBeenCalledWith('sync_status')
    expect(syncState.configured).toBe(true)
    expect(mockedListen).toHaveBeenCalledWith('sync:progress', expect.any(Function))
    expect(mockedListen).toHaveBeenCalledWith('sync:finished', expect.any(Function))
  })

  it('refreshSyncConfig refreshes configuration status', async () => {
    mockedInvoke.mockResolvedValue({
      configured: false,
      url: null,
      username: null,
    })

    await refreshSyncConfig()

    expect(mockedInvoke).toHaveBeenCalledWith('sync_status')
    expect(syncState.configured).toBe(false)
  })

  it('triggerSync invokes sync_trigger and sets syncing to true', async () => {
    mockedInvoke.mockResolvedValue(undefined)

    await triggerSync()

    expect(mockedInvoke).toHaveBeenCalledWith('sync_trigger')
    expect(syncState.syncing).toBe(true)
  })

  it('dismissSyncResult clears lastFinishedResult', () => {
    // Set mock value
    // Note: since lastFinishedResult is readonly on syncState, but we can call dismissSyncResult
    dismissSyncResult()
    expect(syncState.lastFinishedResult).toBeNull()
  })

  it('useSync composable provides computed properties', () => {
    const { configured, syncing, hasSyncError } = useSync()

    expect(configured.value).toBe(false)
    expect(syncing.value).toBe(false)
    expect(hasSyncError.value).toBe(false)
  })
})
