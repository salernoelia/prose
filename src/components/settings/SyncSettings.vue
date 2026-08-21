<script
    setup
    lang="ts"
>
import { ref } from 'vue'
import InputText from 'primevue/inputtext'
import Button from 'primevue/button'
import type { SyncStatusDto } from '../../ipc/types'

defineProps<{
    syncConfig: SyncStatusDto
    syncWorking: boolean
    syncMessage: { text: string; ok: boolean } | null
}>()

const syncUrl = defineModel<string>('url', { default: '' })
const syncUsername = defineModel<string>('username', { default: '' })
const syncPassword = defineModel<string>('password', { default: '' })

const showPassword = ref(false)

const emit = defineEmits<{
    (e: 'save'): void
    (e: 'disconnect'): void
}>()
</script>

<template>
    <div class="flex flex-col gap-6 pt-4 border-t border-(--border-color)">
        <div class="flex items-center justify-between">
            <h2 class="text-xs font-semibold uppercase tracking-wider text-(--text-secondary)">
                WebDAV Cloud Sync
            </h2>
            <span
                class="text-[11px] font-medium px-2.5 py-0.5 rounded-full"
                :class="syncConfig.configured
                    ? 'bg-emerald-100 dark:bg-emerald-950/40 text-emerald-800 dark:text-emerald-400 font-semibold'
                    : 'bg-(--accent-color-light) text-(--text-tertiary)'"
            >
                {{ syncConfig.configured ? 'Connected' : 'Not Configured' }}
            </span>
        </div>

        <div class="flex flex-col gap-4">
            <div class="flex flex-col gap-1.5">
                <label
                    for="sync-url"
                    class="text-xs font-medium text-(--text-secondary)"
                >
                    Server URL
                </label>
                <InputText
                    id="sync-url"
                    v-model="syncUrl"
                    placeholder="https://example.com/remote.php/dav/files/user/"
                    class="w-full text-xs rounded-xl focus-ring-minimal"
                    :disabled="syncWorking"
                />
            </div>

            <div class="flex flex-col gap-1.5">
                <label
                    for="sync-username"
                    class="text-xs font-medium text-(--text-secondary)"
                >
                    Username
                </label>
                <InputText
                    id="sync-username"
                    v-model="syncUsername"
                    placeholder="user"
                    class="w-full text-xs rounded-xl focus-ring-minimal"
                    :disabled="syncWorking"
                />
            </div>

            <div class="flex flex-col gap-1.5">
                <label
                    for="sync-password"
                    class="text-xs font-medium text-(--text-secondary)"
                >
                    Password / App Token
                </label>
                <div class="relative w-full">
                    <input
                        id="sync-password"
                        v-model="syncPassword"
                        :type="showPassword ? 'text' : 'password'"
                        placeholder="••••••••"
                        class="w-full text-xs rounded-xl px-3 py-2 bg-(--bg-card) text-(--text-primary) border border-(--border-color) hover:border-(--border-color-hover) focus:border-(--accent-color) focus:ring-1 focus:ring-(--accent-color) focus:outline-none pr-10 transition-all font-sans shadow-2xs"
                        :disabled="syncWorking"
                    />
                    <button
                        type="button"
                        @click.stop.prevent="showPassword = !showPassword"
                        class="absolute right-2.5 top-1/2 -translate-y-1/2 z-10 flex items-center justify-center text-(--text-secondary) hover:text-(--text-primary) transition-colors p-1 cursor-pointer select-none"
                        :title="showPassword ? 'Hide password' : 'Show password'"
                        :aria-label="showPassword ? 'Hide password' : 'Show password'"
                    >
                        <span class="material-symbols-outlined text-lg select-none leading-none">
                            {{ showPassword ? 'visibility' : 'visibility_off' }}
                        </span>
                    </button>
                </div>
            </div>

            <div
                v-if="syncMessage"
                class="text-xs px-3.5 py-2.5 rounded-xl"
                :class="syncMessage.ok
                    ? 'bg-emerald-50 dark:bg-emerald-950/30 text-emerald-700 dark:text-emerald-400 border border-emerald-200 dark:border-emerald-900'
                    : 'bg-red-50 dark:bg-red-950/30 text-red-700 dark:text-red-400 border border-red-200 dark:border-red-900'"
            >
                {{ syncMessage.text }}
            </div>

            <div class="flex items-center gap-3 pt-2">
                <Button
                    v-if="syncConfig.configured"
                    label="Disconnect"
                    severity="danger"
                    variant="text"
                    :loading="syncWorking"
                    @click="emit('disconnect')"
                    class="px-3.5 w-full py-2 text-xs font-medium text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/20 rounded-full transition-all cursor-pointer"
                />
                <Button
                    label="Save & Connect"
                    :loading="syncWorking"
                    @click="emit('save')"
                    class="px-4 w-full py-2 text-xs font-medium rounded-full border border-(--border-color) bg-(--bg-card) text-(--text-primary) hover:bg-(--accent-color-light) transition-all cursor-pointer focus-ring-minimal shadow-xs"
                />

            </div>
        </div>
    </div>
</template>
