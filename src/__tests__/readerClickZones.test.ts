import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import { mount } from '@vue/test-utils'
import ReaderClickZones from '../components/reader/ReaderClickZones.vue'

function dispatchTouch(target: EventTarget, type: 'touchstart' | 'touchend', x: number, y: number) {
  const event = new Event(type, { bubbles: true, cancelable: true }) as TouchEvent
  Object.defineProperty(event, 'changedTouches', {
    value: [{ screenX: x, screenY: y }],
  })
  target.dispatchEvent(event)
}

describe('ReaderClickZones', () => {
  const originalInnerWidth = window.innerWidth

  beforeEach(() => {
    Object.defineProperty(window, 'innerWidth', { configurable: true, value: 1000 })
  })

  afterEach(() => {
    Object.defineProperty(window, 'innerWidth', { configurable: true, value: originalInnerWidth })
  })

  it('emits prev on quick edge tap outside reader host', () => {
    const wrapper = mount(ReaderClickZones, {
      props: { clickZoneSize: 20 },
    })
    const outside = document.createElement('div')
    document.body.appendChild(outside)

    dispatchTouch(outside, 'touchstart', 40, 100)
    dispatchTouch(outside, 'touchend', 40, 100)

    expect(wrapper.emitted('prev')).toBeTruthy()
    outside.remove()
  })

  it('emits next on quick edge tap outside reader host', () => {
    const wrapper = mount(ReaderClickZones, {
      props: { clickZoneSize: 20 },
    })
    const outside = document.createElement('div')
    document.body.appendChild(outside)

    dispatchTouch(outside, 'touchstart', 970, 100)
    dispatchTouch(outside, 'touchend', 970, 100)

    expect(wrapper.emitted('next')).toBeTruthy()
    outside.remove()
  })

  it('does not emit edge tap turns inside reader host', () => {
    const wrapper = mount(ReaderClickZones, {
      props: { clickZoneSize: 20 },
    })
    const host = document.createElement('div')
    host.setAttribute('data-reader-host', '')
    document.body.appendChild(host)

    dispatchTouch(host, 'touchstart', 20, 100)
    dispatchTouch(host, 'touchend', 20, 100)

    expect(wrapper.emitted('prev')).toBeFalsy()
    expect(wrapper.emitted('next')).toBeFalsy()
    host.remove()
  })

  it('keeps swipe paging behavior', () => {
    const wrapper = mount(ReaderClickZones)
    const outside = document.createElement('div')
    document.body.appendChild(outside)

    dispatchTouch(outside, 'touchstart', 120, 100)
    dispatchTouch(outside, 'touchend', 220, 100)

    expect(wrapper.emitted('prev')).toBeTruthy()
    outside.remove()
  })
})
