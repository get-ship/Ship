<controller>
import { controller } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './ship-agent-controls.sail'

// Each verb dispatches its canonical control event with the agent + state in
// the detail so production can route it to the matching control RPC. Which
// verbs are *visible* is a pure attribute-selector concern (see the .sail);
// the controller only emits.
export default controller(({ agent, state }: ViewScope<Scope>, host) => {

    const emit = (name: string): void => {
      host?.dispatchEvent(
        new CustomEvent(name, {
          detail: { agent: agent(), state: state() },
          bubbles: true,
          composed: true,
        }),
      )
    }
    return {
      start: () => emit('ship-start'),
      steer: () => emit('ship-steer'),
      approve: () => emit('ship-approve'),
      deny: () => emit('ship-deny'),
      rerun: () => emit('ship-rerun'),
      stop: () => emit('ship-stop'),
    }
})
</controller>

<!--
  ship-agent-controls.sail — @ship/agents · the session-control verbs as a bar.
  Context-aware: the canonical verbs surface only when valid for the current state —
    idle      → Start
    active    → Steer · Stop
    suspended → Approve · Deny · Stop
    completed/failed → Rerun
  (Per SPEC; the design sketch reused a "Start" button for terminal states, but the
  spec's verb is Rerun with its own ship-rerun event — implemented as such here.)
  Controller emits ship-start · ship-steer · ship-approve · ship-deny · ship-stop ·
  ship-rerun. Attribute driven (state), tokens only.  Props: state · agent.
-->
<!-- <prop name="agent" type="string" default="" /> -->
<!-- <prop name="state" values="idle|active|suspended|completed|failed" default="active" reflect /> -->

<!-- style block removed: use sx / external CSS -->

<row gap="3" align="center">
  <span>
    <span></span>
    <span data-test="controls-agent">{agent()}</span>
    <span>{state()}</span>
  </span>
  <row align="center" gap="2">
    <button type="button" data-test="verb-start" on:click={start}><svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M8 5v14l11-7z"></path></svg>Start</button>
    <button type="button" data-test="verb-steer" on:click={steer}><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M22 2 11 13M22 2l-7 20-4-9-9-4 20-7z"></path></svg>Steer</button>
    <button type="button" data-test="verb-approve" on:click={approve}><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M20 6 9 17l-5-5"></path></svg>Approve</button>
    <button type="button" data-test="verb-deny" on:click={deny}>Deny</button>
    <button type="button" data-test="verb-rerun" on:click={rerun}><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M3 7v6h6M3 13a9 9 0 1 0 3-7.7L3 8"></path></svg>Rerun</button>
    <button type="button" data-test="verb-stop" on:click={stop}><svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><rect x="6" y="6" width="12" height="12" rx="1.5"></rect></svg>Stop</button>
  </row>
</row>
