import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import StatCard from '../components/stats/StatCard.vue'
import WeeklyActivityChart from '../components/stats/WeeklyActivityChart.vue'
import BookListItem from '../components/library/BookListItem.vue'
import LibraryHeader from '../components/library/LibraryHeader.vue'

describe('Stats Components', () => {
  it('renders StatCard correctly', () => {
    const wrapper = mount(StatCard, {
      props: {
        label: 'Streak',
        value: 5,
        unit: 'days',
        subtitle: 'Best: 10 days',
      },
    })
    expect(wrapper.text()).toContain('Streak')
    expect(wrapper.text()).toContain('5')
    expect(wrapper.text()).toContain('days')
    expect(wrapper.text()).toContain('Best: 10 days')
  })

  it('renders WeeklyActivityChart correctly', () => {
    const bars = [
      { date: '2026-07-20', label: 'Mon', totalSeconds: 600, height: 20, active: true },
      { date: '2026-07-21', label: 'Tue', totalSeconds: 0, height: 4, active: false },
    ]
    const wrapper = mount(WeeklyActivityChart, {
      props: {
        bars,
        todayISO: '2026-07-21',
      },
    })
    expect(wrapper.text()).toContain('This week')
    expect(wrapper.text()).toContain('Mon')
    expect(wrapper.text()).toContain('Tue')
  })
})

describe('Library Components', () => {
  it('renders LibraryHeader correctly and emits events', async () => {
    const wrapper = mount(LibraryHeader, {
      props: {
        configured: true,
        syncing: false,
        hasSyncError: false,
      },
    })
    expect(wrapper.text()).toContain('Library')
    expect(wrapper.text()).toContain('Sync')
    expect(wrapper.text()).toContain('Import')

    const buttons = wrapper.findAll('button')
    await buttons[0].trigger('click')
    expect(wrapper.emitted('sync')).toBeTruthy()

    await buttons[1].trigger('click')
    expect(wrapper.emitted('import')).toBeTruthy()
  })

  it('renders BookListItem correctly and emits events', async () => {
    const entry = {
      book: {
        id: '1',
        title: 'The Great Gatsby',
        author: 'F. Scott Fitzgerald',
        format: 'epub' as const,
        file_path: '/path/gatsby.epub',
        cover: null,
        created_at: 0,
      },
      progress: 0.42,
      lastRead: 100,
      archived: false,
    }

    const wrapper = mount(BookListItem, {
      props: { entry },
    })

    expect(wrapper.text()).toContain('The Great Gatsby')
    expect(wrapper.text()).toContain('F. Scott Fitzgerald')
    expect(wrapper.text()).toContain('42%')

    await wrapper.trigger('click')
    expect(wrapper.emitted('select')).toBeTruthy()
  })
})
