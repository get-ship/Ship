<scope>
  props: {
    label = ''
  }
</scope>

<!--
  <ship-rail-section> — labeled section divider for <ship-rail>.
  Co-located with <ship-rail> in the same directory.

  Prop surface
  ─────────────────────────────────────────────────────────────────────────────
  label  (string, '')  section heading text
  ─────────────────────────────────────────────────────────────────────────────
  Slots: (default) — accepts <ship-rail-item> children for this section
  Parts: heading
  ─────────────────────────────────────────────────────────────────────────────
  CSS custom properties consumed from parent <ship-rail>:
    --rail-label-display  (flex | none)  hides heading when rail is collapsed
  ─────────────────────────────────────────────────────────────────────────────
-->

<span>{label()}</span>
<slot></slot>
