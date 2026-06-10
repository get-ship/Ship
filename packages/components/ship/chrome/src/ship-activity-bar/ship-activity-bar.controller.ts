import { controller } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-activity-bar.sui'

const CONTEXTS = [
  'agents', 'views', 'components', 'pipelines',
  'themes', 'collections', 'artifacts', 'settings',
] as const
type Context = (typeof CONTEXTS)[number]

function isContext(v: string | undefined): v is Context {
  return v != null && (CONTEXTS as readonly string[]).includes(v)
}

// Vertical context switcher. Each item IS a context; clicking one becomes the
// active context (reflected for styling) and emits ship-context-change so the
// host can swap the navigator panel.
export default controller({
  scope: ({ active }: ViewScope<Scope>, host) => {
    function pick(e: Event): void {
      const ctx = (e.currentTarget as HTMLElement | null)?.dataset.ctx
      if (!isContext(ctx)) return
      active.set(ctx)
      host?.dispatchEvent(
        new CustomEvent('ship-context-change', {
          detail: { context: ctx },
          bubbles: true,
          composed: true,
        }),
      )
    }
    return { pick }
  },
})
