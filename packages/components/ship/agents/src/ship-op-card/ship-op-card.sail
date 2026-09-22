<!--
  ship-op-card.sail — @ship/agents · the gated op. ★ authority made visible.
  One op event (tool call / code-mode op) rendered with its name + args + result,
  PLUS the two authority signals the Floki model makes first-class:
    • blast tier  — C0..C3, the op's inherent authority weight (escalating tone)
    • gate decision — allowed · approved · needs-approval · denied (+ running)
  A denial renders as DATA (the reason), never a crash.

  Tokens only. Controller emits ship-toggle {open} on chevron click.
  Props: op · args · result · reason · ms · blast(C0|C1|C2|C3) · decision · open.
-->
<scope>
  props: {
    op = 'op'
    args = ''
    result = ''
    reason = ''
    ms = ''
    blast: 'C0' | 'C1' | 'C2' | 'C3' = 'C0'
    decision: 'allowed' | 'approved' | 'needs-approval' | 'denied' | 'running' = 'allowed'
    open = false
  }
</scope>
<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './ship-op-card.sail'
import { emitToggle } from './host'

export default controller((ctx: ControllerScope<Scope>, host?: Element) => {
  const toggle = (): void => {
    const next = !ctx.open()
    ctx.open.set(next)
    emitToggle(host, next)
  }
  return { toggle }
})
</controller>

<column gap="2">
  <row align="center" gap="2">
    <span data-test="op-name">{op()}</span>
    <badge data-test="op-blast">{blast()}</badge>
    <span>{decision()}</span>
    <if {ms()}>
      <span>{ms()}</span>
    </if>
    <button type="button" data-test="op-toggle" on:click={toggle} label="Toggle args"></button>
  </row>
  <if {open()}>
    <span data-test="op-args">{args()}</span>
  </if>
  <if {result()}>
    <span data-test="op-result">{result()}</span>
  </if>
  <if {reason()}>
    <span data-test="op-reason">{reason()}</span>
  </if>
</column>
