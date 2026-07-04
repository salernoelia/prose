<script
    setup
    lang="ts"
>
import { ref, watch, onMounted, onUnmounted } from 'vue'

const props = defineProps<{
    src: string | null
}>()

const emit = defineEmits<{
    (e: 'close'): void
}>()

const MIN_SCALE = 1
const MAX_SCALE = 5

const scale = ref(1)
const tx = ref(0)
const ty = ref(0)

// Pointer bookkeeping: a single pointer pans, two pointers pinch-zoom.
const pointers = new Map<number, { x: number; y: number }>()
let panStart = { x: 0, y: 0, tx: 0, ty: 0 }
let pinchStartDist = 0
let pinchStartScale = 1

const clamp = (v: number, lo: number, hi: number) => Math.min(Math.max(v, lo), hi)

function reset() {
    scale.value = 1
    tx.value = 0
    ty.value = 0
    pointers.clear()
}

// A fresh image starts fit-to-screen.
watch(() => props.src, reset)

function onWheel(e: WheelEvent) {
    e.preventDefault()
    const next = clamp(scale.value - e.deltaY * 0.002, MIN_SCALE, MAX_SCALE)
    scale.value = next
    if (next === MIN_SCALE) {
        tx.value = 0
        ty.value = 0
    }
}

function dist(a: { x: number; y: number }, b: { x: number; y: number }) {
    return Math.hypot(a.x - b.x, a.y - b.y)
}

function onPointerDown(e: PointerEvent) {
    ;(e.target as HTMLElement).setPointerCapture?.(e.pointerId)
    pointers.set(e.pointerId, { x: e.clientX, y: e.clientY })
    if (pointers.size === 1) {
        panStart = { x: e.clientX, y: e.clientY, tx: tx.value, ty: ty.value }
    } else if (pointers.size === 2) {
        const [a, b] = [...pointers.values()]
        pinchStartDist = dist(a, b)
        pinchStartScale = scale.value
    }
}

function onPointerMove(e: PointerEvent) {
    if (!pointers.has(e.pointerId)) return
    pointers.set(e.pointerId, { x: e.clientX, y: e.clientY })

    if (pointers.size >= 2) {
        const [a, b] = [...pointers.values()]
        const d = dist(a, b)
        if (pinchStartDist > 0) {
            scale.value = clamp((d / pinchStartDist) * pinchStartScale, MIN_SCALE, MAX_SCALE)
        }
        return
    }

    if (scale.value > 1) {
        tx.value = panStart.tx + (e.clientX - panStart.x)
        ty.value = panStart.ty + (e.clientY - panStart.y)
    }
}

function onPointerUp(e: PointerEvent) {
    pointers.delete(e.pointerId)
    if (pointers.size < 2) pinchStartDist = 0
    if (scale.value <= MIN_SCALE) {
        tx.value = 0
        ty.value = 0
    }
}

// Double tap / click toggles between fit and 2x.
function onDoubleClick() {
    if (scale.value > 1) {
        reset()
    } else {
        scale.value = 2
    }
}

function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') emit('close')
}

onMounted(() => window.addEventListener('keydown', onKey))
onUnmounted(() => window.removeEventListener('keydown', onKey))
</script>

<template>
    <Teleport to="body">
        <div
            v-if="src"
            class="fixed inset-0 z-[60] flex items-center justify-center bg-black/90 animate-fade-in select-none touch-none overflow-hidden"
            @click.self="emit('close')"
            @wheel="onWheel"
        >
            <img
                :src="src"
                alt=""
                draggable="false"
                class="max-w-full max-h-full object-contain"
                :style="{
                    transform: `translate(${tx}px, ${ty}px) scale(${scale})`,
                    cursor: scale > 1 ? 'grab' : 'default',
                }"
                @pointerdown="onPointerDown"
                @pointermove="onPointerMove"
                @pointerup="onPointerUp"
                @pointercancel="onPointerUp"
                @dblclick="onDoubleClick"
            />

            <button
                type="button"
                class="fixed right-3 flex items-center justify-center w-10 h-10 rounded-full bg-black/40 text-white/90 hover:text-white active:scale-90 transition-all focus-ring-minimal"
                style="top: calc(0.75rem + env(safe-area-inset-top, 0px))"
                title="Close"
                aria-label="Close image"
                @click="emit('close')"
            >
                <span class="material-symbols-outlined text-xl">close</span>
            </button>
        </div>
    </Teleport>
</template>
