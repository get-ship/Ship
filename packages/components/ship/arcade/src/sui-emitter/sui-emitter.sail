<!--
  <sui-emitter> — declarative particle emitter.

  Pooled, max-bounded. `rate` > 0 emits continuously; otherwise the emitter
  only fires when the stage receives the `on` action (with optional
  `{x, y}` detail to relocate the burst).

  `size` is `"start..end"` (e.g. `"4..0"` shrinks to nothing). `color`
  accepts a theme token name (`accent`, `tone-success-bg`) or hex.
-->
<scope>
  props: {
    kind: 'radial' | 'cone' | 'line' | 'explosion' = 'radial'
    x = 0
    y = 0
    rate = 0
    burst = 16
    lifetime = 600
    spread = 360
    direction = -90
    speed = 120
    'speed-jitter' = 0.3
    gravity = 0
    color = 'accent'
    size: string = '3..0'
    max = 64
    on = ''
  }
</scope>
