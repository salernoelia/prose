<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import Button from 'primevue/button'

const greetMsg = ref('')
const name = ref('')

async function greet() {
  greetMsg.value = await invoke('greet', { name: name.value })
}
</script>

<template>
  <main class="container">
    <Button label="Click" @click="greet" />
  </main>
</template>
