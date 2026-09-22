<!--
  <sui-sprite> — declarative image entity.

  The stage spawns one engine Entity per sui-sprite child it finds at init.
  Sprite-sheet animation: set `frames="<cols>x<rows>"` and `frame-rate` to
  cycle frames per second; `frame` overrides the current cell when
  frame-rate is 0.

  Most gameplay sprites are spawned by the director directly via
  stage.world.spawn(); markup sprites are for level scenery / static
  decorations.
-->
<scope>
  props: {
    src = ''
    x = 0
    y = 0
    z = 0
    vx = 0
    vy = 0
    scale = 1
    rotation = 0
    anchor = 'center'
    tint = ''
    alpha = 1
    frames = ''
    'frame-rate' = 0
    frame = 0
    group = ''
    tag = ''
  }
</scope>
<slot></slot>
