<script setup lang="ts">
import InputText from 'primevue/inputtext'
import Password from 'primevue/password'
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

const emit = defineEmits<{
    (e: 'save'): void
    (e: 'disconnect'): void
}>()
</script>

<template>
    <div class="flex flex-col gap-6 pt-4 border-t border-(--border-color)">
        <div class="flex items-center justify-between">
            <h2 class="text-xs font-semibold uppercase tracking-wider text-(--text-secondary)">
                Sync (WebDAV)
            </h2>
            <span
                class="text-xs font-medium px-2 py-0.5 rounded"
                :class="syncConfig.configured
                    ? 'bg-emerald-100 dark:bg-emerald-950/40 text-emerald-800 dark:text-emerald-400'
                    : 'bg-(--accent-color-light) text-(--text-tertiary)'"
            >
                {{ syncConfig.configured ? 'Connected' : 'Not Configured' }}
            </span>
        </div>

        <div class="flex flex-col gap-4">
            <div class="flex flex-col gap-1.5">
                <label
                    for="sync-url"
                    class="text-xs font-medium uppercase tracking-wider text-(--text-secondary)"
                >
                    Server URL
                </label>
                <InputText
                    id="sync-url"
                    v-model="syncUrl"
                    placeholder="https://example.com/remote.php/dav/files/user/"
                    class="w-full text-sm focus-ring-minimal"
                    :disabled="syncWorking"
                />
            </div>

            <div class="flex flex-col gap-1.5">
                <label
                    for="sync-username"
                    class="text-xs font-medium uppercase tracking-wider text-(--text-secondary)"
                >
                    Username
                </label>
                <InputText
                    id="sync-username"
                    v-model="syncUsername"
                    placeholder="user"
                    class="w-full text-sm focus-ring-minimal"
                    :disabled="syncWorking"
                />
            </div>

            <div class="flex flex-col gap-1.5">
                <label
                    for="sync-password"
                    class="text-xs font-medium uppercase tracking-wider text-(--text-secondary)"
                >
                    Password / App Token
                </label>
                <Password
                    id="sync-password"
                    v-model="syncPassword"
                    :feedback="false"
                    toggleMask
                    placeholder="••••••••"
                    class="w-full text-sm focus-ring-minimal"
                    :disabled="syncWorking"
                />
            </div>

            <div
                v-if="syncMessage"
                class="text-xs px-3 py-2 rounded"
                :class="syncMessage.ok
                    ? 'bg-emerald-50 dark:bg-emerald-950/30 text-emerald-700 dark:text-emerald-400 border border-emerald-200 dark:border-emerald-900'
                    : 'bg-red-50 dark:bg-red-950/30 text-red-700 dark:text-red-400 border border-red-200 dark:border-red-900'"
            >
                {{ syncMessage.text }}
            </div>

            <div class="flex items-center gap-3 pt-2">
                <Button
                    label="Save & Connect"
                    :loading="syncWorking"
                    @click="emit('save')"
                    class="px-4 py-1.5 text-xs font-semibold rounded border border-(--border-color) bg-(--bg-card) text-(--text-primary) hover:bg-(--accent-color-light) transition-all cursor-pointer focus-ring-minimal"
                />
                <Button
                    v-if="syncConfig.configured"
                    label="Disconnect"
                    severity="danger"
                    variant="text"
                    :loading="syncWorking"
                    @click="emit('disconnect')"
                    class="px-3 py-1.5 text-xs font-semibold text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/20 rounded transition-all cursor-pointer"
                />
            </div>
        </div>
    </div>
</template>
