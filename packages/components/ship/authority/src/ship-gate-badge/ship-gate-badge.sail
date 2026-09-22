<scope>
  props: {
    decision: 'allowed' | 'approved' | 'needs-approval' | 'denied' | 'running' = 'allowed'
  }
</scope>
<!--
  <ship-gate-badge> — @ship/authority atom · a gate decision verdict:
  allowed · approved · needs-approval · denied · running. Extracted from op-card so
  the verdict reads identically everywhere it surfaces (op-card, gate-log,
  capability-bar). Attribute-driven, tokens only, no controller, no events.

  The decision reflects to the host attribute (CSS picks the tone + dot) and the
  matching label shows via CSS; `running` pulses (honoring reduced-motion).

  Prop:  decision  allowed | approved | needs-approval | denied | running  (reflect)
  Part:  badge
-->

<span><span></span><span>allowed</span><span>approved</span><span>needs approval</span><span>denied</span><span>running</span></span>
