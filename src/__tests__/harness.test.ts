import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/vue'
import { defineComponent, h } from 'vue'

// Proves the test harness wiring (vitest + jsdom + Vue + Testing Library) works.
// Real component tests replace this as the views land.
const Probe = defineComponent({
  setup: () => () => h('p', 'Prose'),
})

describe('test harness', () => {
  it('renders a Vue component into jsdom', () => {
    render(Probe)
    expect(screen.getByText('Prose')).toBeTruthy()
  })
})
