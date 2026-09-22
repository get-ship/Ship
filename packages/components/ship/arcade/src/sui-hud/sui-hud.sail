<!--
  <sui-hud> — DOM overlay on top of the stage's canvas.

  Hosts regular L1 primitives above the canvas. Descendants with
  `data-bind="<key>"` subscribe to the stage's scope signal of the same key —
  the stage controller wires the binding; the HUD is otherwise inert.

  `align` picks one of nine grid placements; `pointer-events="auto"` opts
  the overlay back into pointer interaction (default is `none` so the
  game still receives clicks behind the HUD).
-->
<scope>
  props: {
    align: 'top' | 'bottom' | 'left' | 'right' | 'center' | 'corner-tl' | 'corner-tr' | 'corner-bl' | 'corner-br' = 'top'
    'z-layer' = 1
    'pointer-events': 'auto' | 'none' = 'none'
  }
</scope>
<slot></slot>
