import { bindHtml, controller, derived, effect, ref } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-harness-blob.sui'
import { HARNESSES, type Harness } from './harness-logos'

export default controller({
  scope: (ctx: ViewScope<Scope>) => {
    const tileEl = ref<HTMLSpanElement>(null)
    const tileLabel = derived<string>(
      () => (HARNESSES[ctx.harness() as Harness] ?? HARNESSES.claude).label,
    )
    return { tileEl, tileLabel }
  },

  mount(state, host) {
    // Set the brand hue on the host element so the tile background +
    // outline read from --blob-hue. Lit version does the same in render().
    const stopHue = effect(() => {
      const meta = HARNESSES[state.harness as Harness] ?? HARNESSES.claude
      host.style.setProperty('--blob-hue', meta.hue)
    })

    // Inject the inline brand SVG into the tile via bindHtml — SUI's
    // analogue to lit's unsafeHTML directive. Re-runs when harness
    // changes.
    const stopSvg = effect(() => {
      const tile = state.tileEl as HTMLSpanElement | null
      if (!tile) return
      return bindHtml(tile, () => {
        const meta = HARNESSES[state.harness as Harness] ?? HARNESSES.claude
        return meta.svg
      })
    })

    return () => {
      stopHue()
      stopSvg()
    }
  },
})
