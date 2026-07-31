<!--
  <ship-watch-badge> — @ship/shared · a tiny connection-status dot + label.

  Presentational atom for surfaces that watch a live stream (jobs, sessions,
  tailing runs). The host passes the connection state via the `state`
  reflected attribute; CSS picks the tone + dot and shows the matching label.
  `state="live"` adds a soft pulse that only fires when the user has not asked
  for reduced motion. `label` overrides the default copy when set — useful
  for surfacing a sub-status ("live · 3s ago") without forking the component.

  Prop:  state   live | connecting | paused | error  (reflect, default live)
  Prop:  label   string                                (reflect, default "")
  Part:  badge · dot · label
-->
<!-- <prop name="state" values="live|connecting|paused|error" default="live" reflect /> -->
<!-- <prop name="label" type="string" default="" reflect /> -->

<!-- style block removed: use sx / external CSS -->

<span>
  <span></span>
  <span>live</span>
  <span>connecting…</span>
  <span>paused</span>
  <span>reconnecting</span>
  <span>{label()}</span>
</span>
