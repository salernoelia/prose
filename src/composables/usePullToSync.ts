import { onBeforeUnmount, onMounted, ref } from 'vue'

/**
 * Pull-to-sync: a native-feeling "drag down from the top to refresh" gesture
 * for the library. It watches touch drags on a scroll container and, once the
 * pull passes a threshold while the container is at the top, fires `onTrigger`
 * (the sync action). The returned `pull`/`armed`/`dragging` refs drive a visual
 * indicator. Mouse and desktop are untouched: only touch events engage.
 */
interface PullToSyncOptions {
  /** Whether a pull may start a sync right now (configured and not already syncing). */
  enabled: () => boolean
  /** Run when the gesture is released past the threshold. */
  onTrigger: () => void
  /** Pull distance, in pixels, that arms the trigger. */
  threshold?: number
}

export function usePullToSync(
  getScrollEl: () => HTMLElement | null,
  options: PullToSyncOptions,
) {
  const threshold = options.threshold ?? 72

  /** Current dampened pull distance in pixels (0 when idle). */
  const pull = ref(0)
  /** True once the pull is far enough to trigger on release. */
  const armed = ref(false)
  /** True while a finger is actively pulling, so transitions can pause. */
  const dragging = ref(false)

  let el: HTMLElement | null = null
  let startX = 0
  let startY = 0
  let tracking = false
  // Direction lock: undecided until the first significant move tells us whether
  // this is a vertical pull (engage) or a horizontal/upward scroll (bail).
  let decided = false

  // Diminishing returns so the pull feels elastic and never runs away.
  function dampen(distance: number): number {
    return Math.min(threshold * 1.8, distance * 0.5)
  }

  function reset(): void {
    tracking = false
    decided = false
    dragging.value = false
    pull.value = 0
    armed.value = false
  }

  function onTouchStart(event: TouchEvent): void {
    if (!el || event.touches.length !== 1) return
    if (el.scrollTop > 0 || !options.enabled()) return
    startX = event.touches[0].clientX
    startY = event.touches[0].clientY
    tracking = true
    decided = false
  }

  function onTouchMove(event: TouchEvent): void {
    if (!tracking || !el) return
    const dx = event.touches[0].clientX - startX
    const dy = event.touches[0].clientY - startY

    if (!decided) {
      // Horizontal intent (e.g. the filter chips row): leave it to the page.
      if (Math.abs(dx) > Math.abs(dy) && Math.abs(dx) > 6) {
        tracking = false
        return
      }
      if (dy > 6) {
        decided = true
        dragging.value = true
      } else {
        if (dy < -2) tracking = false
        return
      }
    }

    // A late scroll (content grew, or momentum) takes over: hand back control.
    if (el.scrollTop > 0) {
      reset()
      return
    }
    if (dy <= 0) {
      pull.value = 0
      armed.value = false
      return
    }

    pull.value = dampen(dy)
    armed.value = pull.value >= threshold
    // Suppress the WebView's own rubber-band overscroll while we own the pull.
    event.preventDefault()
  }

  function onTouchEnd(): void {
    if (!tracking) {
      reset()
      return
    }
    const shouldTrigger = armed.value && options.enabled()
    reset()
    if (shouldTrigger) options.onTrigger()
  }

  onMounted(() => {
    el = getScrollEl()
    if (!el) return
    el.addEventListener('touchstart', onTouchStart, { passive: true })
    el.addEventListener('touchmove', onTouchMove, { passive: false })
    el.addEventListener('touchend', onTouchEnd, { passive: true })
    el.addEventListener('touchcancel', onTouchEnd, { passive: true })
  })

  onBeforeUnmount(() => {
    if (!el) return
    el.removeEventListener('touchstart', onTouchStart)
    el.removeEventListener('touchmove', onTouchMove)
    el.removeEventListener('touchend', onTouchEnd)
    el.removeEventListener('touchcancel', onTouchEnd)
  })

  return { pull, armed, dragging, threshold }
}
