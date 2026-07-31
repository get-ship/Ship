<controller>
import { controller, signal } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './ship-home-dock.sail'

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
export default controller((_ctx: ViewScope<Scope>) => {

    const apps = signal<DockApp[]>([])
    return { apps }
})
</controller>

<!--
  ship-home-dock.sail — @ship/chrome · the desktop launcher dock.

  A centered, glassmorphic tray that composes <ship-dock-app> tiles — one per
  app in the `apps` identity array (each `{ label, icon, href }`). The tile
  owns the lift; this tray owns the glass surface, the hover scale, and the
  tooltip pill. Launch flows through <ship-dock-app>'s bubbling, composed
  `ship-launch { label }` event; the host maps label → href and navigates
  (the dock stays route-agnostic so any desktop can reuse it).

  `apps` is a JS-property (identity) binding — `apps={…}` — not a
  serialized attribute, mirroring <sail-tree>'s `nodes`. The controller
  owns the signal and reflects the host property onto it.

  Tokens only: glass surface from --card + color-mix translucency, --border,
  --shadow-md, --radius-*; pill from --foreground/--background + --font-mono.
-->

<!-- style block removed: use sx / external CSS -->

<nav aria-label="App launcher">
  <for as="app" of={apps()} key={app.href}>
    <span>
      <ship-dock-app label={app.label}>
        <icon name={app.icon} aria-hidden="true"></icon>
      </ship-dock-app>
      <span role="tooltip">{app.label}</span>
    </span>
  </for>
</nav>
