<controller>
import { controller } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './ship-agent-chat.sail'

// <ship-agent-chat> is the conversation shell: identity header, slotted body,
// and a composer. The component is presentation + event-out — it owns no
// message array and never touches the SDK. The host subscribes to
// `ship.agents.session()` (or its equivalent) and slot-renders each turn.
//
// The three events:
//   ship-agent-chat-send     composer submit (button click or Enter key)
//   ship-agent-chat-cancel   Escape on the cmdk surface (a user setting)
//   ship-agent-chat-switch   the Switch affordance — the host routes this to
//                            its own session picker
//
// State reflects: the host flips `state` to 'streaming' / 'awaiting' as the
// turn progresses; the status dot colour + blink fall out of CSS.
export default controller(({ agent, session }: ViewScope<Scope>, host) => {

    function readInput(): HTMLInputElement | null {
      if (!host) return null
      const root = host.shadowRoot ?? host
      return root.querySelector<HTMLInputElement>('[data-test="composer-input"]')
    }

    function emit<T>(type: string, detail: T): void {
      host?.dispatchEvent(
        new CustomEvent(type, { detail, bubbles: true, composed: true }),
      )
    }

    function send(): void {
      const input = readInput()
      const text = (input?.value ?? '').trim()
      if (!text) return
      emit('ship-agent-chat-send', { text, agent: agent(), session: session() })
      if (input) input.value = ''
    }

    function onComposerKey(e: KeyboardEvent): void {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault()
        send()
      } else if (e.key === 'Escape') {
        e.preventDefault()
        emit('ship-agent-chat-cancel', { agent: agent(), session: session() })
      }
    }

    function switchAgent(): void {
      emit('ship-agent-chat-switch', { agent: agent(), session: session() })
    }

    return { send, onComposerKey, switchAgent }
})
</controller>

<!--
  <ship-agent-chat> — @ship/agents · the conversation thread, presentation-agnostic.

  The shell for one session's chat: identity header (agent · session + Switch), a
  scrollable body the turns slot into (sui-message + ship-op-card + ship-approval-prompt),
  and a composer. `surface` = cmdk (centered palette) | drawer (edge sheet) — a user
  SETTING; both toggle/hide to reveal a full canvas. Reconciles with the chrome's
  sui-command-chat (ai mode). Body is slotted.

  Props: surface (cmdk|drawer) · agent · session · state (idle|streaming|awaiting).

  Events (bubble + composed):
    ship-agent-chat-send    { detail: { text, agent, session } } — composer submit
    ship-agent-chat-cancel  { detail: { agent, session } }       — Escape key (cmdk surface)
    ship-agent-chat-switch  { detail: { agent, session } }       — Switch affordance
-->
<!-- <prop name="surface" values="cmdk|drawer" default="cmdk" reflect /> -->
<!-- <prop name="agent" type="string" default="agent" /> -->
<!-- <prop name="session" type="string" default="" /> -->
<!-- <prop name="state" values="idle|streaming|awaiting" default="idle" reflect /> -->

<!-- style block removed: use sx / external CSS -->

<div>
  <span></span>
  <column gap="0">
    <span data-test="agent-name">{agent()}</span>
    <span data-test="session-id">{session()}</span>
  </column>
  <button type="button" data-test="switch" on:click={switchAgent} aria-label="Switch agent">
    <icon name="chevrons-up-down" size="xs"></icon>
    Switch
  </button>
  <span aria-hidden="true">esc</span>
</div>
<column gap="3"><slot></slot></column>
<div>
  <div>
    <input
      type="text"
      data-test="composer-input"
      placeholder="Message the agent…"
      aria-label="Message the agent"
      on:keydown={onComposerKey}
    />
    <button type="button" data-test="composer-send" on:click={send} aria-label="Send">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 2L11 13M22 2l-7 20-4-9-9-4 20-7z"></path></svg>
    </button>
  </div>
</div>