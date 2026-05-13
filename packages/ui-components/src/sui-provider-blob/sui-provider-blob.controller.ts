import { bindHtml, controller, derived, effect, ref } from '@ship/sui-runtime'
import type { Signal } from '@ship/sui-runtime'
import type { Scope } from './sui-provider-blob.sui'
import { PROVIDERS, type Provider } from './provider-logos'

export default controller<Scope>({
  scope: (state) => {
    const s = state as unknown as {
      provider: Signal<Provider>
    }
    const tileEl = ref<HTMLSpanElement>(null)
    const tileLabel = derived<string>(
      () => (PROVIDERS[s.provider()] ?? PROVIDERS.claude).label,
    )
    return { tileEl, tileLabel }
  },

  mount(state, host) {
    const scope = state as unknown as {
      provider: Signal<Provider>
      tileEl: Signal<HTMLSpanElement | null>
    }

    // Set the brand hue on the host element so the tile background +
    // outline read from --blob-hue. Lit version does the same in render().
    const stopHue = effect(() => {
      const meta = PROVIDERS[scope.provider()] ?? PROVIDERS.claude
      host.style.setProperty('--blob-hue', meta.hue)
    })

    // Inject the inline brand SVG into the tile via bindHtml — SUI's
    // analogue to lit's unsafeHTML directive. Re-runs when provider
    // changes.
    const stopSvg = effect(() => {
      const tile = scope.tileEl()
      if (!tile) return
      return bindHtml(tile, () => {
        const meta = PROVIDERS[scope.provider()] ?? PROVIDERS.claude
        return meta.svg
      })
    })

    return () => {
      stopHue()
      stopSvg()
    }
  },
})
