import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/vue'
import PrimeVue from 'primevue/config'
import Library from '../views/LibraryView.vue'
import { initLibraryStore } from '../stores/library'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  convertFileSrc: vi.fn((path) => path),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}))

vi.mock('@tauri-apps/api/path', () => ({
  appDataDir: vi.fn().mockResolvedValue('/app/data'),
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'
const mockedInvoke = vi.mocked(invoke)

describe('Library View Component', () => {
  beforeEach(() => {
    mockedInvoke.mockReset()
    const mockEntries = [
      {
        book: {
          id: '1',
          format: 'epub' as const,
          title: 'Alice in Wonderland',
          author: 'Lewis Carroll',
          cover: null,
        },
        progress: 0.35,
        lastRead: 100,
      },
      {
        book: {
          id: '2',
          format: 'pdf' as const,
          title: 'The Odyssey',
          author: 'Homer',
          cover: null,
        },
        progress: 0.12,
        lastRead: 200,
      },
    ]
    mockedInvoke.mockResolvedValue(mockEntries)
  })

  it('renders the library view and search elements', async () => {
    await initLibraryStore()

    render(Library, {
      global: {
        plugins: [PrimeVue],
      },
    })

    expect(screen.getByText('Library')).toBeTruthy()
    expect(screen.getByPlaceholderText('Search by title or author')).toBeTruthy()
  })
})
