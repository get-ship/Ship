<!--
  <sui-hud> — DOM overlay on top of the stage's canvas.

  Hosts regular SUI primitives (sui-stat, sui-meter, sui-card, etc.) above
  the canvas. Descendants with `data-bind="<key>"` subscribe to the stage's
  scope signal of the same key — the stage controller wires the binding;
  the HUD is otherwise inert.

  `align` picks one of nine grid placements; `pointer-events="auto"` opts
  the overlay back into pointer interaction (default is `none` so the
  game still receives clicks behind the HUD).
-->

<!-- <prop name="align" values="top|bottom|left|right|center|corner-tl|corner-tr|corner-bl|corner-br" default="top" reflect /> -->
<!-- <prop name="z-layer" type="number" default="1" /> -->
<!-- <prop name="pointer-events" values="auto|none" default="none" reflect /> -->

<!-- style block removed: use sx / external CSS -->

<slot></slot>
