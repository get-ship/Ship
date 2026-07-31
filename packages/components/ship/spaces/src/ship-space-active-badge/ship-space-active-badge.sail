<!--
  ship-space-active-badge.sail — the @ship/spaces ACTIVE BADGE.
  A read-only "currently in X" chip reflecting the daemon-global DEFAULT
  POINTER (SpaceService active_id — where context-free invocations route). When
  no space is active (global-only mode: active_id = null), it renders the empty
  state instead. Purely presentational — `name` / `path` are fed by the
  integration layer (List + the active pointer); no controller, no daemon.
  Compiled as <ship-space-active-badge>. Parts: badge · dot · label.
-->
<!-- <prop name="name" type="string" default="" /> -->
<!-- <prop name="path" type="string" default="" /> -->

<!-- style block removed: use sx / external CSS -->

<if cond={name()}>
  <span>
    <span></span>
    <span>in</span>
    <span>{name()}</span>
    <if cond={path()}><span>{path()}</span></if>
  </span>
<else>
  <span>
    <span></span>
    <span>No active space</span>
  </span>
</else></if>
