import { controller, effect, ref, signal } from '@ship/sui-runtime'
import type { Scope } from './sui-toast.sui'

type DismissReason = 'manual' | 'timeout' | 'programmatic'

const TONES_WITH_AUTO_ICON = new Set(['info', 'success', 'warning', 'destructive', 'accent'])

export default controller<Scope>({
  scope: () => {
    const hasSlottedIcon = signal(false)
    const visible = signal(true)
    const iconSlotEl = ref<HTMLSlotElement>(null)
    const titleSlotEl = ref<HTMLSlotElement>(null)
    const actionsSlotEl = ref<HTMLSlotElement>(null)
    const closeBtn = ref<HTMLButtonElement>(null)
    return { hasSlottedIcon, visible, iconSlotEl, titleSlotEl, actionsSlotEl, closeBtn }
  },

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
      // Flip `visible` to false; bindIf runs `out:fade` on the .toast
      // wrapper. The out-transition's duration (180ms — see template)
      // gates when we tear the host down.
      const visSignal = scope.visible as unknown as ReturnType<typeof signal<boolean>>
      visSignal.set(false)
      await new Promise<void>((resolve) => setTimeout(resolve, 200))
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

    const hasSlottedIconSig = scope.hasSlottedIcon as unknown as ReturnType<typeof signal<boolean>>

    const slotIsFilled = (slot: HTMLSlotElement): boolean =>
      slot.assignedNodes({ flatten: true }).some((n) => {
        if (n.nodeType === Node.ELEMENT_NODE) return true
        return (n.textContent?.trim().length ?? 0) > 0
      })

    const iconSlotEl = scope.iconSlotEl() as HTMLSlotElement | null
    if (iconSlotEl) {
      const onIconSlotChange = () => hasSlottedIconSig.set(slotIsFilled(iconSlotEl))
      iconSlotEl.addEventListener('slotchange', onIconSlotChange)
      hasSlottedIconSig.set(slotIsFilled(iconSlotEl))
    }

    const titleSlotEl = scope.titleSlotEl() as HTMLSlotElement | null
    const actionsSlotEl = scope.actionsSlotEl() as HTMLSlotElement | null

    if (titleSlotEl) {
      const onTitleChange = () =>
        host.toggleAttribute('data-no-title', !slotIsFilled(titleSlotEl))
      titleSlotEl.addEventListener('slotchange', onTitleChange)
      host.toggleAttribute('data-no-title', !slotIsFilled(titleSlotEl))
    }

    if (actionsSlotEl) {
      const onActionsChange = () =>
        host.toggleAttribute('data-no-actions', !slotIsFilled(actionsSlotEl))
      actionsSlotEl.addEventListener('slotchange', onActionsChange)
      host.toggleAttribute('data-no-actions', !slotIsFilled(actionsSlotEl))
    }

    const stopIconAttrEffect = effect(() => {
      const tone = scope.tone()
      const slotted = scope.hasSlottedIcon()
      const hasAuto = TONES_WITH_AUTO_ICON.has(tone)
      host.toggleAttribute('data-no-icon', !hasAuto && !slotted)
    })

    const stopCloseEffect = effect(() => {
      const btn = scope.closeBtn()
      if (!btn) return
      const onClick = () => dismiss('manual')
      btn.addEventListener('click', onClick)
      return () => btn.removeEventListener('click', onClick)
    })

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
      stopIconAttrEffect()
      stopCloseEffect()
    }
  },
})
