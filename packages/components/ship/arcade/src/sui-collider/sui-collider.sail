<!--
  <sui-collider> — declares a collider on the parent <sui-sprite> / <sui-shape>.

  Read once by the stage at init. `group` is this entity's collision group;
  `collides-with` is a comma-separated list of groups this entity tests
  against. Set `trigger` to skip physical response (events only).

  Default dimensions:
    aabb   — falls back to parent's width/height (or 16x16 for sprites
             until the texture loads)
    circle — falls back to parent's radius (or 8)
-->
<scope>
  props: {
    kind: 'aabb' | 'circle' = 'aabb'
    width = 0
    height = 0
    radius = 0
    'offset-x' = 0
    'offset-y' = 0
    group = ''
    'collides-with' = ''
    trigger = false
  }
</scope>
