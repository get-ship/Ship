<!--
  <sui-shape> — declarative geometric entity.

  Like <sui-sprite> but drawn by Pixi Graphics. `fill` and `stroke` accept
  theme token names (`accent`, `surface-fg`, `tone-success-bg`) so the
  shape colors shift with the active theme.

  Shapes are useful for retro-arcade art (rects + circles + polygons) and
  for placeholder gameplay sprites before art assets exist.
-->
<scope>
  props: {
    kind: 'rect' | 'circle' | 'polygon' | 'line' = 'rect'
    width = 10
    height = 10
    radius = 5
    points = ''
    x = 0
    y = 0
    z = 0
    vx = 0
    vy = 0
    scale = 1
    rotation = 0
    alpha = 1
    fill = 'foreground'
    stroke = ''
    'stroke-width' = 1
    group = ''
    tag = ''
  }
</scope>
<slot></slot>
