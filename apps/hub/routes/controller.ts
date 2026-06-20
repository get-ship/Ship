// Hub home controller — the simplest possible controller. The
// `name` signal exists so the template has a signal to read,
// which triggers the compiler to emit a `hydrate()` function
// on the view's setup return. Without any signal read, the
// compiler doesn't emit hydrate and the runtime's mount() has
// nothing to call.
//
// Slice 2 — uses the canonical `controller({...})` form (no
// explicit generic) so the `SC` parameter is inferred from the
// scope factory's return. The synthetic controller's `__def.scope`
// recovery (G4 fix) then recovers the exact scope shape
// (`{ name: Signal<string> }`) instead of the empty view
// `Scope` the legacy `controller<Scope>({...})` form pinned.
import { controller, signal } from '@ship/sui-runtime'

export default controller({
  scope: () => ({
    name: signal('Hub'),
  }),
})
