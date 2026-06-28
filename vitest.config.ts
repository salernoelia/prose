import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'

// Self-contained config for unit tests. The Vue plugin transforms `.vue`
// single-file components; jsdom provides a DOM for component rendering.
export default defineConfig({
  plugins: [vue()],
  test: {
    environment: 'jsdom',
    include: ['src/**/*.{test,spec}.ts'],
  },
})
