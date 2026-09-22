<!--
  <ship-agent-chat> — @ship/agents · the conversation thread, presentation-agnostic.

  The shell for one session's chat: identity header (agent · session + Switch), a
  scrollable body the turns slot into (message + ship-op-card + ship-approval-prompt),
  and a composer. `surface` = cmdk (centered palette) | drawer (edge sheet) — a user
  SETTING; both toggle/hide to reveal a full canvas. Reconciles with the chrome's
  command-chat (ai mode). Body is slotted.

  Props: surface (cmdk|drawer) · agent · session · state (idle|streaming|awaiting).

  Events (bubble + composed):
    ship-agent-chat-send    { detail: { text, agent, session } } — composer submit
    ship-agent-chat-cancel  { detail: { agent, session } }       — Escape key (cmdk surface)
    ship-agent-chat-switch  { detail: { agent, session } }       — Switch affordance
-->
<scope>
  props: {
    surface: 'cmdk' | 'drawer' = 'cmdk'
    agent = 'agent'
    session = ''
    state: 'idle' | 'streaming' | 'awaiting' = 'idle'
  }
</scope>
<controller>
import type { ControllerScope } from '@ship/sail-runtime'
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
export default controller((ctx: ControllerScope<Scope>, host?: Element) => {
    function readInput(): { value: string } | null {
      if (!host) return null
      const root = host.shadowRoot ?? host
      return root.querySelector('[data-test="composer-input"]') as { value: string } | null
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
      emit('ship-agent-chat-send', { text, agent: ctx.agent(), session: ctx.session() })
      if (input) input.value = ''
    }

    function onComposerKey(e: KeyboardEvent): void {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault()
        send()
      } else if (e.key === 'Escape') {
        e.preventDefault()
        emit('ship-agent-chat-cancel', { agent: ctx.agent(), session: ctx.session() })
      }
    }

    function switchAgent(): void {
      emit('ship-agent-chat-switch', { agent: ctx.agent(), session: ctx.session() })
    }

    return { send, onComposerKey, switchAgent }
})
</controller>

<div>
  <span></span>
  <column gap="0">
    <span data-test="agent-name">{agent()}</span>
    <span data-test="session-id">{session()}</span>
  </column>
  <button type="button" data-test="switch" on:click={switchAgent} label="Switch agent">
    <icon name="chevrons-up-down" size="xs"></icon>
    Switch
  </button>
  <span>esc</span>
</div>
<column gap="3"><slot></slot></column>
<div>
  <div>
    <input
      type="text"
      data-test="composer-input"
      placeholder="Message the agent…"
      label="Message the agent"
      on:keydown={onComposerKey}
    />
    <button type="button" data-test="composer-send" on:click={send} label="Send">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 2L11 13M22 2l-7 20-4-9-9-4 20-7z"></path></svg>
    </button>
  </div>
</div>
