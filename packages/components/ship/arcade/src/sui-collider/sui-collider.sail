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

<!-- <prop name="kind" values="aabb|circle" default="aabb" reflect /> -->
<!-- <prop name="width" type="number" default="0" /> -->
<!-- <prop name="height" type="number" default="0" /> -->
<!-- <prop name="radius" type="number" default="0" /> -->
<!-- <prop name="offset-x" type="number" default="0" /> -->
<!-- <prop name="offset-y" type="number" default="0" /> -->
<!-- <prop name="group" type="string" default="" reflect /> -->
<!-- <prop name="collides-with" type="string" default="" reflect /> -->
<!-- <prop name="trigger" type="bool" default="false" reflect /> -->

<!-- style block removed: use sx / external CSS -->
