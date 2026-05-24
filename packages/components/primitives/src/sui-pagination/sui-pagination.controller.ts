import { controller, signal, ref, effect } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-pagination.sui'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => ({
    totalPages: signal(1),
    info: signal(''),
    prevBtn: ref<HTMLButtonElement>(null),
    nextBtn: ref<HTMLButtonElement>(null),
  }),

  mount(state, host) {
    const stopEffect = effect(() => {
      const p = state.page
      const t = state.total
      const pp = state.perPage
      const tp = t > 0 ? Math.ceil(t / pp) : 1
      state.totalPages = tp
      if (t > 0) {
        const start = (p - 1) * pp + 1
        const end = Math.min(p * pp, t)
        state.info = `Page ${p} of ${tp} (${start}–${end} of ${t})`
      } else {
        state.info = `Page ${p} of ${tp}`
      }
    })

    const prevBtn = state.prevBtn as HTMLButtonElement | null
    const nextBtn = state.nextBtn as HTMLButtonElement | null

    const onPrev = () => {
      const p = state.page
      if (p > 1) {
        const np = p - 1
        state.page = np
        host.dispatchEvent(
          new CustomEvent('sui-page', {
            detail: { page: np },
            bubbles: true,
            composed: true,
          }),
        )
      }
    }

    const onNext = () => {
      const p = state.page
      const tp = state.totalPages
      if (p < tp) {
        const np = p + 1
        state.page = np
        host.dispatchEvent(
          new CustomEvent('sui-page', {
            detail: { page: np },
            bubbles: true,
            composed: true,
          }),
        )
      }
    }

    prevBtn?.addEventListener('click', onPrev)
    nextBtn?.addEventListener('click', onNext)

    return () => {
      stopEffect()
      prevBtn?.removeEventListener('click', onPrev)
      nextBtn?.removeEventListener('click', onNext)
    }
  },
})
