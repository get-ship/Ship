<controller>
import { controller } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './ship-agent-slot.sail'

// Presentation is reflected-attribute driven; the only behavior is Save, which
// graduates the injected content into a durable artifact. The controller emits
// ship-save with the slot label so production can route it to the Save RPC.
export default controller(({ label }: ViewScope<Scope>, host) => {

    function save(): void {
      host?.dispatchEvent(
        new CustomEvent('ship-save', {
          detail: { label: label() },
          bubbles: true,
          composed: true,
        }),
      )
    }
    return { save }
})
</controller>

<!--
  ship-agent-slot.sail — @ship/agents · a region an agent can inject into. ★ the primitive.
  The general building block behind Agent Canvas — a bounded area the agent renders
  (morphs) UI into. NOT coupled to the canvas feature: drop one in a view, a card, a
  drawer. Content is slotted. States: empty (awaiting) · rendering (agent injecting) ·
  filled (interactive — Save on hover) · saved. Save graduates the injected content
  into a durable artifact (emits ship-save). Attribute-driven, tokens only.
  Props: state · label.
-->
<!-- <prop name="state" values="empty|rendering|filled|saved" default="filled" reflect /> -->
<!-- <prop name="label" type="string" default="" /> -->

<!-- style block removed: use sx / external CSS -->

<div>
  <column gap="2" align="center">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="3" y="3" width="18" height="18" rx="2" stroke-dasharray="3 3"></rect><path d="M12 8v8M8 12h8"></path></svg>
    <span data-test="slot-label">{label()}</span>
  </column>
  <div>
    <span><span></span>rendering</span>
    <div></div><div></div><div></div>
  </div>
  <div>
    <button type="button" data-test="slot-save" on:click={save}><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"></path><path d="M17 21v-8H7v8M7 3v5h8"></path></svg>Save</button>
    <span><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M20 6L9 17l-5-5"></path></svg>saved</span>
  </div>
  <div><slot></slot></div>
</div>
