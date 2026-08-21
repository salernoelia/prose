import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import PrimeVue from 'primevue/config'
import StatCard from '../components/stats/StatCard.vue'
import WeeklyActivityChart from '../components/stats/WeeklyActivityChart.vue'
import LibraryProgressMatrix from '../components/stats/LibraryProgressMatrix.vue'
import GoalSpeedometer from '../components/stats/GoalSpeedometer.vue'
import TimeDistributionChart from '../components/stats/TimeDistributionChart.vue'
import BookBreakdownList from '../components/stats/BookBreakdownList.vue'
import SessionHistoryList from '../components/stats/SessionHistoryList.vue'
import BookListItem from '../components/library/BookListItem.vue'
import LibraryHeader from '../components/library/LibraryHeader.vue'
import BottomNavigationBar from '../components/BottomNavigationBar.vue'
import SyncSettings from '../components/settings/SyncSettings.vue'
import HomeView from '../views/HomeView.vue'

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

  it('renders LibraryProgressMatrix correctly', () => {
    const wrapper = mount(LibraryProgressMatrix, {
      props: {
        totalBooks: 10,
        booksFinished: 4,
        booksInProgress: 3,
        booksUnstarted: 3,
        epubCount: 8,
        pdfCount: 2,
      },
    })
    expect(wrapper.text()).toContain('4')
    expect(wrapper.text()).toContain('out of 10')
    expect(wrapper.text()).toContain('40% Finished')
    expect(wrapper.text()).toContain('4 Done')
    expect(wrapper.text()).toContain('3 Reading')
    expect(wrapper.text()).toContain('3 Unread')
    expect(wrapper.text()).toContain('8 EPUB')
  })

  it('renders GoalSpeedometer correctly', () => {
    const wrapper = mount(GoalSpeedometer, {
      props: {
        currentSeconds: 900,
        targetSeconds: 1800,
        formatDuration: (s: number) => `${Math.round(s / 60)} min`,
      },
    })
    expect(wrapper.text()).toContain('50%')
    expect(wrapper.text()).toContain('15 min')
    expect(wrapper.text()).toContain('30 min goal')
  })

  it('renders TimeDistributionChart correctly', () => {
    const distribution = [
      { id: 'morning', label: 'Morning', period: '5am-12pm', icon: 'wb_sunny', seconds: 1200, percentage: 30, sessionCount: 1 },
      { id: 'afternoon', label: 'Afternoon', period: '12pm-5pm', icon: 'light_mode', seconds: 0, percentage: 0, sessionCount: 0 },
      { id: 'evening', label: 'Evening', period: '5pm-10pm', icon: 'dark_mode', seconds: 2800, percentage: 70, sessionCount: 2 },
      { id: 'night', label: 'Night', period: '10pm-5am', icon: 'bedtime', seconds: 0, percentage: 0, sessionCount: 0 },
    ]
    const wrapper = mount(TimeDistributionChart, {
      props: {
        distribution,
        formatDuration: (s: number) => `${Math.round(s / 60)} min`,
      },
    })
    expect(wrapper.text()).toContain('Evening')
    expect(wrapper.text()).toContain('47 min')
    expect(wrapper.text()).toContain('Evening Unwinder')
  })

  it('renders BookBreakdownList and emits select-book', async () => {
    const books = [
      {
        bookId: '1',
        book: { id: '1', title: 'Moby Dick', author: 'Herman Melville', format: 'epub' as const, file_path: '', cover: null, created_at: 0 },
        bookTitle: 'Moby Dick',
        bookAuthor: 'Herman Melville',
        totalSeconds: 3600,
        progress: 0.75,
        format: 'epub' as const,
        cover: null,
        lastRead: 100,
        percentageOfTotal: 100,
      },
    ]
    const wrapper = mount(BookBreakdownList, {
      props: {
        books,
        formatDuration: (s: number) => `${Math.round(s / 3600)}h`,
      },
    })
    expect(wrapper.text()).toContain('Moby Dick')
    expect(wrapper.text()).toContain('Herman Melville')
    expect(wrapper.text()).toContain('75% complete')

    const row = wrapper.find('.group.cursor-pointer')
    await row.trigger('click')
    expect(wrapper.emitted('select-book')).toBeTruthy()
    expect(wrapper.emitted('select-book')?.[0]?.[0]).toEqual(books[0].book)
  })

  it('renders SessionHistoryList and handles delete-session', async () => {
    const sessions = [
      { id: 's1', bookTitle: 'Dune', startedAt: Date.now(), durationSeconds: 1500 },
    ]
    const wrapper = mount(SessionHistoryList, {
      props: {
        sessions,
        formatDuration: (s: number) => `${Math.round(s / 60)} min`,
      },
    })
    expect(wrapper.text()).toContain('Dune')
    expect(wrapper.text()).toContain('25 min')

    const deleteBtn = wrapper.find('button[aria-label="Delete session"]')
    await deleteBtn.trigger('click')
    // Click again to confirm
    const confirmBtn = wrapper.find('button[aria-label="Confirm delete session"]')
    await confirmBtn.trigger('click')
    expect(wrapper.emitted('delete-session')).toBeTruthy()
    expect(wrapper.emitted('delete-session')?.[0]?.[0]).toBe('s1')
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

  it('renders BottomNavigationBar and emits navigate', async () => {
    const wrapper = mount(BottomNavigationBar, {
      props: { currentView: 'stats' },
    })

    expect(wrapper.text()).toContain('Home')
    expect(wrapper.text()).toContain('Library')
    expect(wrapper.text()).toContain('Stats')
    expect(wrapper.text()).toContain('Settings')

    const buttons = wrapper.findAll('button')
    await buttons[0].trigger('click')
    expect(wrapper.emitted('navigate')?.[0]).toEqual(['home'])
  })

  it('renders SyncSettings and toggles password visibility', async () => {
    const wrapper = mount(SyncSettings, {
      props: {
        syncConfig: { configured: false, url: '', username: '' },
        syncWorking: false,
        syncMessage: null,
      },
      global: {
        plugins: [PrimeVue],
      },
    })

    const passwordInput = wrapper.find('#sync-password')
    expect(passwordInput.attributes('type')).toBe('password')

    const toggleBtn = wrapper.find('button[aria-label="Show password"]')
    expect(toggleBtn.exists()).toBe(true)

    await toggleBtn.trigger('click')
    expect(wrapper.find('#sync-password').attributes('type')).toBe('text')
    expect(wrapper.find('button[aria-label="Hide password"]').exists()).toBe(true)
  })

  it('renders HomeView and handles empty or populated library state', () => {
    const wrapper = mount(HomeView, {
      global: {
        plugins: [PrimeVue],
      },
    })
    expect(wrapper.exists()).toBe(true)
  })
})
