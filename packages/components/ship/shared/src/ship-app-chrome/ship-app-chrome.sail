<!--
  <ship-app-chrome> — @ship/shared · the per-app root layout wrapper.

  RUNTIME-FREE, purely presentational. A chrome wrapper for a route LAYOUT
  (routes/_layout.sail → prefix '/', the longest-prefix match covering the
  index route AND every nested route). Pairs with the bundler's root-layout
  discovery (`crates/deck/deck-bundler/src/routes.rs::discover_layouts`).

  Composes:
    <ship-nav-top>  the filled top bar (brand, breadcrumbs, theme picker,
                    account menu, etc.) — owned by @ship/chrome (it needs the
                    runtime for breadcrumb routing + account state, so it stays
                    a runtime pack; this wrapper only gives it a place to live).
    <slot name="route" />  the route content — the runtime's `mountRouter`
                            (packages/deck/route/src/mount-router.ts::
                            buildMountTarget) inserts a <div slot="route"> here
                            via the chrome chain, and `mount()` projects the
                            route view into that div through this named slot.

  This component owns NO state and imports NOTHING — it has no controller and
  never reaches for @ship/sail-runtime, the SDK, or the contract (enforced by
  `bun run guard`). It is a layout assembler: the bar PLUS the named slot the
  router fills.

  Why a wrapper, not just `<ship-nav-top>` in the layout file? The
  `extract_layout_tag` codegen reads the FIRST `<ship-*>` element of the
  `_layout.sui` and emits it as the LAYOUTS `tag`; the runtime then
  `document.createElement(tag)`s that element. The wrapper IS the chrome. A
  naked `<ship-nav-top>` would mount the bar but never receive the route
  content — the bar has no `<slot name="route">`, by design (it composes the
  slotless `<navbar>` primitive). This wrapper stitches the two.

  Layout: `position: fixed; inset: 0` so the chrome fills the viewport exactly
  (bypasses the browser's default 8px body margin) and is immune to
  content-driven growth. Inner column flex: the bar is fixed-height; the main
  region takes the remaining height via `flex:1`.

  Slots: `route` (router fills this) and `command-bar` (forwarded through to
  `<ship-nav-top slot="command-bar">`). Any consuming layout that wants a
  trigger pill in the top bar slots its command-bar host into this wrapper and
  the forwarding slot chain carries it through the shadow boundaries:
  `_layout` → `ship-app-chrome` → `ship-nav-top` → `.cmd-slot`.
-->

<!-- style block removed: use sx / external CSS -->

<ship-nav-top>
  <slot name="command-bar" slot="command-bar"></slot>
</ship-nav-top>
<main>
  <slot name="route"></slot>
</main>
