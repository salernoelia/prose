<script
    setup
    lang="ts"
>
import { onMounted, ref } from 'vue'
import { useSettings } from '../composables/useSettings'
import { syncConfigure, syncDisconnect, syncStatus } from '../ipc/sync'
import { refreshSyncConfig } from '../stores/sync'
import type { SyncStatusDto } from '../ipc/types'
import {
    AppearanceSettings,
    TypographySettings,
    SyncSettings,
    ReaderBehaviorSettings,
} from '../components/settings'

const {
    settings,
    loaded,
    theme,
    fontFamily,
    fontSize,
    lineHeight,
    margin,
    textAlign,
    clickZoneSize,
    translationLanguage,
} = useSettings()

const syncConfig = ref<SyncStatusDto>({ configured: false, url: null, username: null })
const syncUrl = ref('')
const syncUsername = ref('')
const syncPassword = ref('')
const syncWorking = ref(false)
const syncMessage = ref<{ text: string; ok: boolean } | null>(null)

onMounted(async () => {
    try {
        syncConfig.value = await syncStatus()
        if (syncConfig.value.url) syncUrl.value = syncConfig.value.url
        if (syncConfig.value.username) syncUsername.value = syncConfig.value.username
    } catch {
        // Not configured yet
    }
})

async function handleSyncSave() {
    if (!syncUrl.value.trim() || !syncUsername.value.trim() || !syncPassword.value) {
        syncMessage.value = { text: 'URL, username and password are required.', ok: false }
        return
    }
    syncWorking.value = true
    syncMessage.value = null
    try {
        await syncConfigure(syncUrl.value.trim(), syncUsername.value.trim(), syncPassword.value)
        syncConfig.value = await syncStatus()
        await refreshSyncConfig()
        syncPassword.value = ''
        syncMessage.value = { text: 'Connected successfully.', ok: true }
    } catch (e: unknown) {
        const msg = e && typeof e === 'object' && 'message' in e ? String((e as { message: string }).message) : String(e)
        syncMessage.value = { text: msg, ok: false }
    } finally {
        syncWorking.value = false
    }
}

async function handleSyncDisconnect() {
    syncWorking.value = true
    syncMessage.value = null
    try {
        await syncDisconnect()
        syncConfig.value = { configured: false, url: null, username: null }
        await refreshSyncConfig()
        syncUrl.value = ''
        syncUsername.value = ''
        syncPassword.value = ''
        syncMessage.value = { text: 'Disconnected.', ok: true }
    } finally {
        syncWorking.value = false
    }
}
</script>

<template>
    <div class="w-full animate-fade-in font-serif pb-12">
        <header class="pb-6 pt-4 border-b border-(--border-color) dark:border-white/20 mb-8 flex flex-wrap items-end justify-between gap-3">
            <div>
                <h1 class="text-2xl lg:text-4xl font-semibold tracking-tight text-(--text-primary) font-serif">
                    Settings
                </h1>
            </div>
        </header>

        <div
            v-if="loaded"
            class="flex flex-col gap-8"
        >
            <AppearanceSettings
                v-model:theme="theme"
                v-model:fontFamily="fontFamily"
                :settings="settings"
            />

            <TypographySettings
                v-model:fontSize="fontSize"
                v-model:lineHeight="lineHeight"
                v-model:margin="margin"
                v-model:textAlign="textAlign"
            />

            <ReaderBehaviorSettings
                v-model:clickZoneSize="clickZoneSize"
                v-model:translationLanguage="translationLanguage"
            />

            <SyncSettings
                v-model:url="syncUrl"
                v-model:username="syncUsername"
                v-model:password="syncPassword"
                :syncConfig="syncConfig"
                :syncWorking="syncWorking"
                :syncMessage="syncMessage"
                @save="handleSyncSave"
                @disconnect="handleSyncDisconnect"
            />
        </div>

        <div
            v-else
            class="flex flex-col items-center justify-center py-16 gap-3"
        >
            <div class="w-6 h-6 rounded-full border border-(--border-color) border-t-(--accent-color) animate-spin">
            </div>
            <p class="text-xs text-(--text-secondary) font-medium">Loading settings...</p>
        </div>
    </div>
</template>
