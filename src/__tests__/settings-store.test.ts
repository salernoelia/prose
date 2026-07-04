import { describe, it, expect, vi, beforeEach } from 'vitest'
import { initSettingsStore, settingsState, updateSettings } from '../stores/settings'
import { useSettings } from '../composables/useSettings'

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

describe('Settings Store and Composable', () => {
  beforeEach(() => {
    mockedInvoke.mockReset()
    mockedListen.mockReset()
  })

  it('initSettingsStore seeds state and listens for changes', async () => {
    const mockSettings = {
      schemaVersion: 1,
      theme: 'sepia' as const,
      fontFamily: 'Georgia',
      fontSize: 20,
      lineHeight: 1.6,
      margin: 1.5,
      textAlign: 'left' as const,
    }
    mockedInvoke.mockResolvedValue(mockSettings)

    await initSettingsStore()

    expect(mockedInvoke).toHaveBeenCalledWith('settings_get')
    expect(settingsState.settings).toEqual(mockSettings)
    expect(settingsState.loaded).toBe(true)
  })

  it('updateSettings patches settings and updates state', async () => {
    const updatedSettings = {
      schemaVersion: 1,
      theme: 'dark' as const,
      fontFamily: 'Georgia',
      fontSize: 20,
      lineHeight: 1.6,
      margin: 1.5,
      textAlign: 'left' as const,
    }
    mockedInvoke.mockResolvedValue(updatedSettings)

    await updateSettings({ theme: 'dark' })

    expect(mockedInvoke).toHaveBeenCalledWith('settings_patch', { patch: { theme: 'dark' } })
    expect(settingsState.settings.theme).toBe('dark')
  })

  it('useSettings composable provides computed properties', () => {
    const { theme, fontSize } = useSettings()

    expect(theme.value).toBe('dark')
    expect(fontSize.value).toBe(20)
  })
})
