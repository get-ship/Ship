import { controller, signal, derived, unwrap } from '@ship/sui-runtime'
import { bindHtml } from '@ship/ui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: ({ canvasList, route }) => {
    const firstSlug = derived(() => {
      const list = unwrap(canvasList as any)
      return Array.isArray(list) ? (list[0]?.slug ?? '') : ''
    })
    const activeSlug = signal((route as any)().query?.doc || '')
    const docEl = signal<HTMLElement | null>(null)
    return { activeSlug, docEl, firstSlug }
  },
  on: {
    selectDoc: ({ state, navigate }, slug: string) => {
      state.activeSlug = slug
      navigate('/canvas?doc=' + slug)
    },
  },
  mount: ({ state }) => {
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
