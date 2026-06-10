import { controller } from '@ship/sui-runtime'
import type { Scope } from './sui-toast-host.sui'

type NotifyTone = 'info' | 'success' | 'warning' | 'destructive'

interface NotifyDetail {
  message?: string
  description?: string
  tone?: NotifyTone
  duration?: number
  actionHtml?: string
}

interface ToastContainerLike extends HTMLElement {
  show(opts: {
    tone?: string
    duration?: number
    title?: string
    description?: string
    actionHtml?: string
  }): HTMLElement
}

export default controller<Scope>({
  mount(state, host) {

    const onNotify = (e: Event): void => {
      const detail = ((e as CustomEvent).detail ?? {}) as NotifyDetail
      const container = state.$signals.containerEl() as ToastContainerLike | null
      if (!container || typeof container.show !== 'function') return
      container.show({
        tone: detail.tone,
        duration: detail.duration,
        title: detail.message,
        description: detail.description,
        actionHtml: detail.actionHtml,
      })
    }

    const target: EventTarget =
      host.closest('sui-shell, ship-shell') ??
      host.ownerDocument?.body ??
      host
    target.addEventListener('ship:notify', onNotify)

    return () => {
      target.removeEventListener('ship:notify', onNotify)
    }
  },
})
