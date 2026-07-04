<script
    setup
    lang="ts"
>
import { onMounted, ref } from 'vue'
import { useSettings } from '../composables/useSettings'
import { TRANSLATION_LANGUAGES } from '../lib/externalLookup'
import { syncConfigure, syncDisconnect, syncStatus } from '../ipc/sync'
import { refreshSyncConfig } from '../stores/sync'
import type { SyncStatusDto } from '../ipc/types'
import Select from 'primevue/select'
import Slider from 'primevue/slider'
import InputText from 'primevue/inputtext'
import Password from 'primevue/password'
import Button from 'primevue/button'
import type { Theme, TextAlign } from '../ipc/types'

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

const translationOptions = [...TRANSLATION_LANGUAGES]

const alignOptions: { value: TextAlign; icon: string; label: string }[] = [
    { value: 'left', icon: 'format_align_left', label: 'Left' },
    { value: 'justify', icon: 'format_align_justify', label: 'Justify' },
    { value: 'center', icon: 'format_align_center', label: 'Center' },
    { value: 'right', icon: 'format_align_right', label: 'Right' },
]

const themeOptions = [
    { label: 'Light', value: 'light' as Theme },
    { label: 'Paper', value: 'paper' as Theme },
    { label: 'Dark', value: 'dark' as Theme },
    { label: 'OLED Black', value: 'oled' as Theme },
    { label: 'Night', value: 'night' as Theme },
    { label: 'Sepia', value: 'sepia' as Theme },
    { label: 'Sepia Dark', value: 'sepia-dark' as Theme },
    { label: 'E-Ink Light', value: 'eink' as Theme },
    { label: 'E-Ink Dark', value: 'eink-dark' as Theme },
]

const fontOptions = [
    { label: 'Georgia', value: 'Georgia' },
    { label: 'Literata', value: 'Literata' },
    { label: 'Inter', value: 'Inter' },
    { label: 'Outfit', value: 'Outfit' },
]

// --- Sync ---

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
        // not configured yet
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
    <div class="w-full animate-fade-in">
        <!-- Typography-driven Header (No Icons) -->
        <header class="pb-6">
            <h1 class="text-xl lg:text-3xl font-semibold tracking-tight text-(--text-primary)">Settings</h1>
        </header>

        <!-- Form Controls (No Icons, Minimal Labels) -->
        <div
            v-if="loaded"
            class="flex flex-col gap-6"
        >
            <!-- Theme Selection -->
            <div class="flex flex-col gap-1.5">
                <label
                    for="theme-select"
                    class="text-xs font-medium uppercase tracking-wider text-(--text-secondary)"
                >
                    Theme
                </label>
                <Select
                    id="theme-select"
                    v-model="theme"
                    :options="themeOptions"
                    optionLabel="label"
                    optionValue="value"
                    class="w-full focus-ring-minimal"
                />
            </div>

            <!-- Font Family Selection -->
            <div class="flex flex-col gap-1.5">
                <label
                    for="font-family-select"
                    class="text-xs font-medium uppercase tracking-wider text-(--text-secondary)"
                >
                    Typeface
                </label>
                <Select
                    id="font-family-select"
                    v-model="fontFamily"
                    :options="fontOptions"
                    optionLabel="label"
                    optionValue="value"
                    class="w-full focus-ring-minimal"
                />
            </div>

            <!-- Typography Preview -->
            <div class="mt-4 flex flex-col gap-1.5">
                <span class="text-xs font-medium uppercase tracking-wider text-(--text-tertiary)">Preview</span>
                <div
                    class="overflow-hidden border border-(--border-color) rounded-lg bg-(--bg-card) shadow-inner"
                    :style="{ height: '300px' }"
                >
                    <div
                        class="h-full overflow-hidden select-none p-6"
                        :style="{
                            fontFamily: settings.fontFamily,
                            fontSize: settings.fontSize + 'px',
                            lineHeight: settings.lineHeight,
                            paddingLeft: settings.margin * 12 + 'px',
                            paddingRight: settings.margin * 12 + 'px',
                        }"
                    >
                        <h2 class="font-semibold mb-2 text-[1.1em] tracking-tight">
                            Chapter I: Down the Rabbit-Hole
                        </h2>
                        <p class="text-left text-[0.95em]">
                            Alice was beginning to get very tired of sitting by her sister on the bank, and of
                            having nothing to do: once or twice she had peeped into the book her sister was
                            reading, but it had no pictures or conversations in it, “and what is the use of a
                            book,” thought Alice “without pictures or conversations?”
                        </p>
                    </div>
                </div>
            </div>

            <!-- Font Size Slider -->
            <div class="flex flex-col gap-1.5">
                <div
                    class="flex justify-between items-center text-xs font-medium uppercase tracking-wider text-(--text-secondary)">
                    <label for="font-size-slider">Size</label>
                    <span class="text-(--text-tertiary)">{{ fontSize }}px</span>
                </div>
                <div class="py-2">
                    <Slider
                        id="font-size-slider"
                        v-model="fontSize"
                        :min="12"
                        :max="48"
                        class="w-full focus-ring-minimal"
                    />
                </div>
            </div>

            <!-- Line Height Slider -->
            <div class="flex flex-col gap-1.5">
                <div
                    class="flex justify-between items-center text-xs font-medium uppercase tracking-wider text-(--text-secondary)">
                    <label for="line-height-slider">Spacing</label>
                    <span class="text-(--text-tertiary)">{{ lineHeight.toFixed(1) }}x</span>
                </div>
                <div class="py-2">
                    <Slider
                        id="line-height-slider"
                        v-model="lineHeight"
                        :min="1.0"
                        :max="3.0"
                        :step="0.1"
                        class="w-full focus-ring-minimal"
                    />
                </div>
            </div>

            <!-- Margin Slider -->
            <div class="flex flex-col gap-1.5">
                <div
                    class="flex justify-between items-center text-xs font-medium uppercase tracking-wider text-(--text-secondary)">
                    <label for="margin-slider">Margin</label>
                    <span class="text-(--text-tertiary)">{{ margin.toFixed(1) }}x</span>
                </div>
                <div class="py-2">
                    <Slider
                        id="margin-slider"
                        v-model="margin"
                        :min="0.5"
                        :max="3.0"
                        :step="0.1"
                        class="w-full focus-ring-minimal"
                    />
                </div>
            </div>

            <!-- Text Alignment -->
            <div class="flex flex-col gap-1.5">
                <div
                    class="flex justify-between items-center text-xs font-medium uppercase tracking-wider text-(--text-secondary)">
                    <label>Alignment</label>
                </div>
                <div class="flex items-center gap-1.5">
                    <button
                        v-for="opt in alignOptions"
                        :key="opt.value"
                        type="button"
                        @click="textAlign = opt.value"
                        :class="[
                            'flex-1 flex items-center justify-center h-9 rounded-md border transition-colors focus-ring-minimal',
                            textAlign === opt.value
                                ? 'border-(--accent-color) text-(--accent-color) bg-(--accent-color)/10'
                                : 'border-(--border-color) text-(--text-secondary) hover:text-(--text-primary)',
                        ]"
                        :title="opt.label"
                        :aria-label="opt.label"
                        :aria-pressed="textAlign === opt.value"
                    >
                        <span class="material-symbols-outlined text-base">{{ opt.icon }}</span>
                    </button>
                </div>
            </div>

            <!-- Click Zone Slider -->
            <div class="flex flex-col gap-1.5">
                <div
                    class="flex justify-between items-center text-xs font-medium uppercase tracking-wider text-(--text-secondary)">
                    <label for="click-zone-slider">Page-Turn Zone</label>
                    <span class="text-(--text-tertiary)">{{ clickZoneSize }}%</span>
                </div>
                <div class="py-2">
                    <Slider
                        id="click-zone-slider"
                        v-model="clickZoneSize"
                        :min="10"
                        :max="45"
                        class="w-full focus-ring-minimal"
                    />
                </div>
            </div>

            <!-- Translation Target Language -->
            <div class="flex flex-col gap-1.5">
                <label
                    for="translation-select"
                    class="text-xs font-medium uppercase tracking-wider text-(--text-secondary)"
                >
                    Translate To
                </label>
                <Select
                    id="translation-select"
                    v-model="translationLanguage"
                    :options="translationOptions"
                    optionLabel="label"
                    optionValue="value"
                    filter
                    class="w-full focus-ring-minimal"
                />
            </div>

            <!-- Sync -->
            <div class="flex flex-col gap-4 pt-4 pb-8 border-t border-(--border-color)">
                <div class="flex items-center justify-between">
                    <span class="text-xs font-medium uppercase tracking-wider text-(--text-secondary)">
                        WebDAV Sync
                    </span>
                    <span
                        v-if="syncConfig.configured"
                        class="text-xs font-medium text-green-600 dark:text-green-400"
                    >
                        Connected
                    </span>
                </div>

                <div class="flex flex-col gap-3">
                    <div class="flex flex-col gap-1.5">
                        <label
                            for="sync-url"
                            class="text-xs font-medium text-(--text-secondary)"
                        >Server URL</label>
                        <InputText
                            id="sync-url"
                            v-model="syncUrl"
                            placeholder="https://dav.example.com/remote.php/dav/files/user"
                            class="w-full text-sm"
                            :disabled="syncWorking"
                        />
                    </div>

                    <div class="flex flex-col gap-1.5">
                        <label
                            for="sync-username"
                            class="text-xs font-medium text-(--text-secondary)"
                        >Username</label>
                        <InputText
                            id="sync-username"
                            v-model="syncUsername"
                            placeholder="username"
                            class="w-full text-sm"
                            :disabled="syncWorking"
                        />
                    </div>

                    <div class="flex flex-col gap-1.5">
                        <label
                            for="sync-password"
                            class="text-xs font-medium text-(--text-secondary)"
                        >Password</label>
                        <Password
                            id="sync-password"
                            v-model="syncPassword"
                            :feedback="false"
                            toggle-mask
                            :placeholder="syncConfig.configured ? 'Leave blank to keep existing' : 'password'"
                            class="w-full text-sm"
                            input-class="w-full"
                            :disabled="syncWorking"
                        />
                    </div>

                    <p
                        v-if="syncMessage"
                        class="text-xs"
                        :class="syncMessage.ok ? 'text-green-800 dark:text-green-600' : 'text-red-500'"
                    >
                        {{ syncMessage.text }}
                    </p>

                    <div class="flex gap-2 pt-1">
                        <Button
                            :label="syncWorking ? 'Connecting...' : syncConfig.configured ? 'Update' : 'Connect'"
                            :loading="syncWorking"
                            :disabled="syncWorking"
                            class="flex-1 bg-green-800 dark:bg-green-600"
                            @click="handleSyncSave"
                        />
                        <Button
                            v-if="syncConfig.configured"
                            label="Disconnect"
                            severity="secondary"
                            :disabled="syncWorking"
                            @click="handleSyncDisconnect"
                            class="bg-red-800 dark:bg-red-700"
                        />
                    </div>
                </div>
            </div>

        </div>

        <!-- Loading State -->
        <div
            v-else
            class="flex flex-col items-center justify-center py-16 gap-3"
        >
            <div class="w-6 h-6 rounded-full border border-(--border-color) border-t-(--accent-color) animate-spin">
            </div>
            <p class="text-xs text-(--text-secondary) font-medium">Loading</p>
        </div>
    </div>
</template>
