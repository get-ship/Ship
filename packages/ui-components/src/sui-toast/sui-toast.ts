import { controller, effect, signal } from '@ship/ui-runtime'
import type { Scope } from './sui-toast.sui'

const TONE_ICON: Record<string, string> = {
  info: '<circle cx="12" cy="12" r="9"/><path d="M12 8h.01"/><path d="M11 12h1v4h1"/>',
  success: '<circle cx="12" cy="12" r="9"/><path d="m8.5 12.5 2.5 2.5 4.5-5"/>',
  warning:
    '<path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0Z"/><line x1="12" x2="12" y1="9" y2="13"/><line x1="12" x2="12.01" y1="17" y2="17"/>',
  destructive:
    '<circle cx="12" cy="12" r="9"/><line x1="15" x2="9" y1="9" y2="15"/><line x1="9" x2="15" y1="9" y2="15"/>',
  accent: '<path d="M12 2 3 14h7l-1 8 10-12h-7l1-8Z"/>',
}

type DismissReason = 'manual' | 'timeout' | 'programmatic'

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope

    let timer: ReturnType<typeof setTimeout> | null = null
    let remaining = 0
    let startedAt = 0
    let removed = false

    if (!host.hasAttribute('tabindex')) host.tabIndex = 0

    function clearTimer() {
      if (timer) { clearTimeout(timer); timer = null }
    }

    function scheduleDismiss() {
      clearTimer()
      const dur = scope.duration()
      if (dur > 0) {
        remaining = dur
        startedAt = Date.now()
        timer = setTimeout(() => dismiss('timeout'), dur)
        host.style.setProperty('--toast-duration', `${dur}ms`)
      }
    }

    function dismiss(reason: DismissReason = 'manual') {
      if (removed) return
      removed = true
      clearTimer()
      host.dispatchEvent(new CustomEvent('sui-dismiss', {
        detail: { reason },
        bubbles: true,
        composed: true,
      }))
      host.remove()
    }

    async function close(reason: DismissReason = 'manual') {
      if (removed) return
      const prefersReduce =
        typeof window !== 'undefined' &&
        window.matchMedia?.('(prefers-reduced-motion: reduce)').matches
      if (prefersReduce) {
        dismiss(reason)
        return
      }
      clearTimer()
      host.setAttribute('closing', '')
      await new Promise<void>((resolve) => {
        const onEnd = () => {
          host.removeEventListener('animationend', onEnd)
          resolve()
        }
        host.addEventListener('animationend', onEnd)
        setTimeout(() => resolve(), 260)
      })
      dismiss(reason)
    }

    const onPause = () => {
      if (!timer) return
      const elapsed = Date.now() - startedAt
      remaining = Math.max(0, remaining - elapsed)
      clearTimer()
      host.toggleAttribute('data-paused', true)
    }

    const onResume = () => {
      if (removed || scope.duration() <= 0) return
      if (remaining <= 0) return
      host.toggleAttribute('data-paused', false)
      startedAt = Date.now()
      timer = setTimeout(() => dismiss('timeout'), remaining)
    }

    const onKeydown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && scope.dismissible()) {
        e.stopPropagation()
        dismiss('manual')
      }
    }

    host.addEventListener('mouseenter', onPause)
    host.addEventListener('mouseleave', onResume)
    host.addEventListener('focusin', onPause)
    host.addEventListener('focusout', onResume)
    host.addEventListener('keydown', onKeydown)

    scheduleDismiss()

    // Auto-icon setup
    const hasSlottedIcon = signal(false)
    let autoIconSvg: SVGSVGElement | null = null

    const iconSlotEl = (scope as unknown as Record<string, () => unknown>).iconSlotEl?.() as HTMLSlotElement | null
    if (iconSlotEl) {
      autoIconSvg = document.createElementNS('http://www.w3.org/2000/svg', 'svg') as SVGSVGElement
      autoIconSvg.setAttribute('viewBox', '0 0 24 24')
      autoIconSvg.setAttribute('fill', 'none')
      autoIconSvg.setAttribute('stroke', 'currentColor')
      autoIconSvg.setAttribute('stroke-width', '2')
      autoIconSvg.setAttribute('stroke-linecap', 'round')
      autoIconSvg.setAttribute('stroke-linejoin', 'round')
      autoIconSvg.setAttribute('aria-hidden', 'true')
      iconSlotEl.appendChild(autoIconSvg)

      const onIconSlotChange = () => {
        const filled = iconSlotEl.assignedNodes({ flatten: true }).some((n) => {
          if (n.nodeType === Node.ELEMENT_NODE) return true
          return (n.textContent?.trim().length ?? 0) > 0
        })
        hasSlottedIcon.set(filled)
      }
      iconSlotEl.addEventListener('slotchange', onIconSlotChange)
    }

    // Slot detection for title and actions
    const titleSlotEl = (scope as unknown as Record<string, () => unknown>).titleSlotEl?.() as HTMLSlotElement | null
    const actionsSlotEl = (scope as unknown as Record<string, () => unknown>).actionsSlotEl?.() as HTMLSlotElement | null

    if (titleSlotEl) {
      const onTitleChange = () => {
        const filled = titleSlotEl.assignedNodes({ flatten: true }).some((n) => {
          if (n.nodeType === Node.ELEMENT_NODE) return true
          return (n.textContent?.trim().length ?? 0) > 0
        })
        host.toggleAttribute('data-no-title', !filled)
      }
      titleSlotEl.addEventListener('slotchange', onTitleChange)
      // Initial state
      host.toggleAttribute('data-no-title', true)
    }

    if (actionsSlotEl) {
      const onActionsChange = () => {
        const filled = actionsSlotEl.assignedNodes({ flatten: true }).some((n) => {
          if (n.nodeType === Node.ELEMENT_NODE) return true
          return (n.textContent?.trim().length ?? 0) > 0
        })
        host.toggleAttribute('data-no-actions', !filled)
      }
      actionsSlotEl.addEventListener('slotchange', onActionsChange)
      host.toggleAttribute('data-no-actions', true)
    }

    // Reactive: update auto-icon SVG and data-no-icon based on tone and slot state
    const stopIconEffect = effect(() => {
      const tone = scope.tone()
      const slotted = hasSlottedIcon()
      const paths = TONE_ICON[tone]

      if (autoIconSvg) {
        if (paths && !slotted) {
          autoIconSvg.innerHTML = paths
          autoIconSvg.hidden = false
        } else {
          autoIconSvg.hidden = true
        }
      }

      const hasAutoIcon = !!paths && !slotted
      host.toggleAttribute('data-no-icon', !hasAutoIcon && !slotted)
    })

    // Close button click — re-query inside effect since the button is conditional
    const stopCloseEffect = effect(() => {
      const btn = (scope as unknown as Record<string, () => unknown>).closeBtn?.() as HTMLButtonElement | null
      if (!btn) return
      const onClick = () => dismiss('manual')
      btn.addEventListener('click', onClick)
      return () => btn.removeEventListener('click', onClick)
    })

    // Expose public methods
    const h = host as HTMLElement & { dismiss: typeof dismiss; close: typeof close }
    h.dismiss = dismiss
    h.close = close

    return () => {
      removed = true
      clearTimer()
      host.removeEventListener('mouseenter', onPause)
      host.removeEventListener('mouseleave', onResume)
      host.removeEventListener('focusin', onPause)
      host.removeEventListener('focusout', onResume)
      host.removeEventListener('keydown', onKeydown)
      stopIconEffect()
      stopCloseEffect()
    }
  },
})
