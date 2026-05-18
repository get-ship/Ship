import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-metachip.sui'

// Compose `role="group"` + `aria-label="<label>: <value>"` on the host
// so the chip reads as a single "key colon value" phrase instead of two
// adjacent inline spans (one with the key, one with the value). Without
// this, screen readers traverse the two slots independently, dropping
// the implicit label-value relationship the visual presentation has.
//
// The aria-label is rebuilt whenever the `label`/`value` props change,
// or when slot content overrides them. Slot-overrides win for visual
// rendering but the prop attributes carry the SR-only text — authors
// who slot rich content (icons, links) should set the props anyway so
// the chip stays announced sensibly.
export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope
    host.setAttribute('role', 'group')
    const stop = effect(() => {
      const k = scope.label()?.trim() ?? ''
      const v = scope.value()?.trim() ?? ''
      const text = k && v ? `${k}: ${v}` : k || v
      if (text) {
        host.setAttribute('aria-label', text)
      } else {
        host.removeAttribute('aria-label')
      }
    })
    return () => stop()
  },
})
