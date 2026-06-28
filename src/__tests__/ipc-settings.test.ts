import { describe, it, expect, vi, beforeEach } from 'vitest'
import { getSettings, patchSettings } from '../ipc/settings'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'
const mockedInvoke = vi.mocked(invoke)

describe('settings IPC wrappers', () => {
  beforeEach(() => {
    mockedInvoke.mockReset()
  })

  it('getSettings invokes the correct command', async () => {
    const fakeSettings = {
      schemaVersion: 1,
      theme: 'light' as const,
      fontFamily: 'Literata',
      fontSize: 18,
      lineHeight: 1.5,
      margin: 1,
    }
    mockedInvoke.mockResolvedValue(fakeSettings)

    const result = await getSettings()

    expect(mockedInvoke).toHaveBeenCalledWith('settings_get')
    expect(result).toEqual(fakeSettings)
  })

  it('patchSettings sends the patch under the correct key', async () => {
    const patch = { theme: 'dark' as const }
    const expected = {
      schemaVersion: 1,
      theme: 'dark' as const,
      fontFamily: 'Literata',
      fontSize: 18,
      lineHeight: 1.5,
      margin: 1,
    }
    mockedInvoke.mockResolvedValue(expected)

    const result = await patchSettings(patch)

    expect(mockedInvoke).toHaveBeenCalledWith('settings_patch', { patch })
    expect(result).toEqual(expected)
  })
})
