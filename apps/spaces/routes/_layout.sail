<!--
  Spaces root layout — every route under `apps/spaces/routes/` is wrapped in
  <ship-space-shell>, the config-driven shell chrome.

  The bundler's `discover_layouts` reads this file as the ROOT layout (sibling
  of `routes/[appId]/page.sui`, prefix `/`) and emits
  `{ pathPrefix: '/', tag: 'ship-space-shell' }` into the LAYOUTS table. The
  runtime's longest-prefix match treats `/` as "every pathname", so the shell
  wraps every space route.

  `extract_layout_tag` reads the FIRST `<ship-*>` element here as the layout
  tag — so <ship-space-shell> IS the chrome. `mountRouter` →
  `buildMountTarget` inserts a `<div slot="route">` into the shell element and
  mounts the active route into it; the shell's `<slot name="route">` (the view
  outlet) projects it. The shell owns all chrome composition (nav/rail/dock,
  cmdk + search, edit overlay) from config — this file just names it.
-->
<ship-space-shell>
  <slot name="route" />
</ship-space-shell>
