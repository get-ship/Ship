import { controller, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-home-dock.sui'

// One launcher entry. `href` is the route the host navigates to when the
// tile's `ship-launch` reaches it; the dock itself never reads it.
export interface DockApp {
  label: string
  icon: string
  href: string
}

// <ship-home-dock> — the desktop launcher tray. Presentational over a single
// identity prop: `apps`. It does NOT navigate (route-agnostic, so any desktop
// reuses it); <ship-dock-app> emits `ship-launch { label }` (bubbles, composed)
// and the host maps that label to a route. The `apps` array is a JS-property
// binding reflected onto the scope signal in `mount`, mirroring <sui-tree-view>.
export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const apps = signal<DockApp[]>([])
    return { apps }
  },

  mount(state, host) {
    const init = (host as { apps?: DockApp[] }).apps
    if (init != null) state.apps = init

    Object.defineProperty(host, 'apps', {
      configurable: true,
      get: () => state.apps,
      set: (v: DockApp[]) => { state.apps = v },
    })
  },
})
