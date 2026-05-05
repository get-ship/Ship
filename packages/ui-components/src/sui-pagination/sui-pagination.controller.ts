import { controller, signal, ref, effect } from '@ship/ui-runtime'
import type { Scope } from './sui-pagination.sui'

export default controller<Scope>({
  scope: () => ({
    totalPages: signal(1),
    info: signal(''),
    prevBtn: ref<HTMLButtonElement>(null),
    nextBtn: ref<HTMLButtonElement>(null),
  }),

  mount(state, host) {
    const scope = state as unknown as Scope

    const stopEffect = effect(() => {
      const p = scope.page()
      const t = scope.total()
      const pp = scope.perPage()
      const tp = t > 0 ? Math.ceil(t / pp) : 1
      scope.totalPages.set(tp)
      if (t > 0) {
        const start = (p - 1) * pp + 1
        const end = Math.min(p * pp, t)
        scope.info.set(`Page ${p} of ${tp} (${start}–${end} of ${t})`)
      } else {
        scope.info.set(`Page ${p} of ${tp}`)
      }
    })

    const prevBtn = scope.prevBtn() as HTMLButtonElement | null
    const nextBtn = scope.nextBtn() as HTMLButtonElement | null

    const onPrev = () => {
      const p = scope.page()
      if (p > 1) {
        const np = p - 1
        scope.page.set(np)
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
      const p = scope.page()
      const tp = scope.totalPages()
      if (p < tp) {
        const np = p + 1
        scope.page.set(np)
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
