import { controller, signal, derived, live, ref, unwrap, bindHtml } from '@ship/sui-runtime'
import { route as routeFactory } from '@ship/sui-runtime/route'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: ({ canvasList, ship }) => {
    const route = routeFactory()
    const firstSlug = derived(() => {
      const list = unwrap(canvasList as any)
      return Array.isArray(list) ? (list[0]?.slug ?? '') : ''
    })
    const activeSlug = signal(route.query.doc() || '')
    const docEl = ref<HTMLElement>(null)
    const canvasDoc = live(() => ship.canvas.doc({ slug: activeSlug() }))
    return { activeSlug, docEl, firstSlug, canvasDoc }
  },
  on: {
    selectDoc: ({ state, navigate }, slug: string) => {
      state.activeSlug = slug
      navigate('/canvas?doc=' + slug)
    },
  },
  mount: (state) => {
    const list = unwrap((state as Record<string, unknown>).canvasList as any)
    if (!state.activeSlug && Array.isArray(list) && list.length > 0) {
      state.activeSlug = list[0].slug
    }
    const el = state.docEl as HTMLElement | null
    if (el) {
      bindHtml(el, () => {
        const doc = unwrap((state as Record<string, unknown>).canvasDoc as any)
        return (doc as { html?: string } | null)?.html ?? ''
      })
    }
  },
})
