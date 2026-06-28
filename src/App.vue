<script setup lang="ts">
import { watchEffect } from 'vue'
import { useSettings } from './composables/useSettings'
import SettingsView from './views/Settings.vue'

const { theme, loaded } = useSettings()

watchEffect(() => {
  if (!loaded.value) return

  const root = document.documentElement
  root.classList.remove('dark', 'sepia')
  
  if (theme.value === 'dark') {
    root.classList.add('dark')
  } else if (theme.value === 'sepia') {
    root.classList.add('sepia')
  }
})
</script>

<template>
  <main class="flex min-h-screen items-center justify-center p-4">
    <SettingsView />
  </main>
</template>

<style>
html.dark {
  background-color: #121212;
  color: #ffffff;
}
html.sepia {
  background-color: #f4ecd8;
  color: #5c4033;
}
</style>
