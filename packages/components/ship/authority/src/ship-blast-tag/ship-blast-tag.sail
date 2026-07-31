<!--
  <ship-blast-tag> — @ship/authority atom · an op's blast tier (C0–C3): its
  inherent authority weight, escalating tone (C0 muted → C1 info → C2 warning →
  C3 destructive). Extracted from op-card so op-card, capability-bar, gate-log and
  the rest of the pack share ONE atom for the tier vocabulary.

  Presentational: no controller, no events. The tier reflects to the host
  attribute (CSS picks the tone) and the matching C# label shows via CSS — so the
  atom renders correctly from markup alone, with no scope.

  Prop:  tier  C0 | C1 | C2 | C3   (reflect)
  Part:  tag
-->
<!-- <prop name="tier" values="C0|C1|C2|C3" default="C0" reflect /> -->

<!-- style block removed: use sx / external CSS -->

<span><span>C0</span><span>C1</span><span>C2</span><span>C3</span></span>
