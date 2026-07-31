<!--
  <ship-rail-item> — nav item for <ship-rail>.
  Co-located with <ship-rail> in the same directory.

  Prop surface
  ─────────────────────────────────────────────────────────────────────────────
  href    (string, '')        navigation target
  label   (string, '')        visible label text
  icon    (string, '')        sui-icon name
  active  (bool, false, reflect)  current-page state (set by shell after nav)
  badge   (string, '')        optional count badge; hidden when empty
  disabled (bool, false, reflect)  non-interactive state
  ─────────────────────────────────────────────────────────────────────────────
  Parts: item, icon-wrap, label, badge
  ─────────────────────────────────────────────────────────────────────────────
  CSS custom properties consumed from parent <ship-rail>:
    --rail-label-display  (flex | none)    hides label when rail is collapsed
    --rail-item-justify   (flex-start | center)  centers icon when collapsed
  These are set on <ship-rail>'s ship-rail-item and cascade to children.
  ─────────────────────────────────────────────────────────────────────────────
  ARIA: aria-current="page" when active
  ─────────────────────────────────────────────────────────────────────────────
-->

<!-- <prop name="href" type="string" default="" /> -->
<!-- <prop name="label" type="string" default="" /> -->
<!-- <prop name="icon" type="string" default="" /> -->
<!-- <prop name="active" type="bool" default="false" reflect /> -->
<!-- <prop name="badge" type="string" default="" /> -->
<!-- <prop name="disabled" type="bool" default="false" reflect /> -->

<!-- style block removed: use sx / external CSS -->

<a bind:this={itemEl}
  href={href() || '#'}
  aria-current={active() ? 'page' : null}
  aria-disabled={disabled() ? 'true' : null}
  title={label()}
>
  <span><icon name={icon()} size="sm"></icon></span>
  <span>{label()}</span>
  <span bind:this={badgeEl}>{badge()}</span>
</a>
