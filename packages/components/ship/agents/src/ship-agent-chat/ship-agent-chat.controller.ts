import { controller } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-agent-chat.sui'

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
export default controller({
  scope: ({ agent, session }: ViewScope<Scope>, host) => {
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
  },
})
