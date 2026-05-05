import { bindHtml, controller, effect, signal } from '@ship/ui-runtime'
import type { Scope } from './sui-toast-container.sui'

type ToastPosition =
  | 'top-left' | 'top-center' | 'top-right'
  | 'bottom-left' | 'bottom-center' | 'bottom-right'

const SLIDE_FROM: Record<ToastPosition, 'top' | 'bottom'> = {
  'top-left': 'top',
  'top-center': 'top',
  'top-right': 'top',
  'bottom-left': 'bottom',
  'bottom-center': 'bottom',
  'bottom-right': 'bottom',
}

interface ShowToastOptions {
  tone?: string
  size?: string
  duration?: number
  dismissible?: boolean
  showProgress?: boolean
  title?: string
  description?: string
  actionHtml?: string
}

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope

    // ARIA defaults
    if (!host.hasAttribute('role')) host.setAttribute('role', 'region')
    if (!host.hasAttribute('aria-label')) host.setAttribute('aria-label', 'Notifications')
    if (!host.hasAttribute('aria-live')) host.setAttribute('aria-live', 'polite')

    const slotVersion = signal(0)
    const defaultSlotEl = scope.defaultSlotEl()

    const getToasts = (): HTMLElement[] => {
      if (defaultSlotEl) {
        return (defaultSlotEl as HTMLSlotElement)
          .assignedElements()
          .filter((e) => e.tagName.toLowerCase() === 'sui-toast') as HTMLElement[]
      }
      return Array.from(host.querySelectorAll(':scope > sui-toast')) as HTMLElement[]
    }

    const stopSlotEffect = effect(() => {
      slotVersion()
      const pos = scope.position() as ToastPosition
      const maxVis = scope.maxVisible()
      const dir = SLIDE_FROM[pos] ?? 'bottom'

      const toasts = getToasts()
      for (const t of toasts) {
        t.setAttribute('data-slide-from', dir)
      }

      if (maxVis > 0 && toasts.length > maxVis) {
        const overflow = toasts.length - maxVis
        for (let i = 0; i < overflow; i++) {
          const victim = toasts[i] as HTMLElement & {
            close?: (r: string) => void
            dismiss?: (r: string) => void
          }
          if (typeof victim.close === 'function') void victim.close('programmatic')
          else victim.dismiss?.('programmatic')
        }
      }
    })

    const onSlotChange = () => slotVersion.set(slotVersion() + 1)
    if (defaultSlotEl) {
      defaultSlotEl.addEventListener('slotchange', onSlotChange)
    }

    function show(opts: ShowToastOptions = {}): HTMLElement {
      const doc = host.ownerDocument ?? document
      const t = doc.createElement('sui-toast') as HTMLElement & {
        dismiss?: (r: string) => void
        close?: (r: string) => void
      }
      if (opts.tone) t.setAttribute('tone', opts.tone)
      if (opts.size) t.setAttribute('size', opts.size)
      if (typeof opts.duration === 'number') t.setAttribute('duration', String(opts.duration))
      if (opts.dismissible === false) t.removeAttribute('dismissible')
      else t.setAttribute('dismissible', '')
      if (opts.showProgress) t.setAttribute('show-progress', '')
      if (opts.title) {
        const span = doc.createElement('span')
        span.setAttribute('slot', 'title')
        span.textContent = opts.title
        t.appendChild(span)
      }
      if (opts.description) {
        t.appendChild(doc.createTextNode(opts.description))
      }
      if (opts.actionHtml) {
        const wrap = doc.createElement('span')
        wrap.setAttribute('slot', 'actions')
        const html = opts.actionHtml
        bindHtml(wrap, () => html)
        t.appendChild(wrap)
      }
      host.appendChild(t)
      return t
    }

    ;(host as HTMLElement & { show: typeof show }).show = show

    return () => {
      stopSlotEffect()
      if (defaultSlotEl) {
        defaultSlotEl.removeEventListener('slotchange', onSlotChange)
      }
    }
  },
})
