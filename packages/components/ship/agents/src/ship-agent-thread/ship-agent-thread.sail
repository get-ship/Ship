<controller>
import { controller, derived, signal } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './ship-agent-thread.sail'

// ── Message contract ──────────────────────────────────────────────────
// The shape this thread renders. It is also the contract the eventual
// `ship.chat.messages({ session_id })` SDK stream is expected to emit —
// when the SDK gap (below) closes, its item type must match this so the
// wire-up stays mechanical.
export interface Message {
  id: string
  role: 'user' | 'assistant' | 'system' | 'tool'
  content_type: 'text' | 'code' | 'tool-call' | 'citation' | 'image'
  text: string
  timestamp: string // ISO 8601
  status?: 'sending' | 'sent' | 'error'
}

// `<sui-message-list>` consumes its own item shape (`content` /
// `contentType`, camelCase). This wrapper owns `Message[]` (the SDK
// contract above) and maps it onto the primitive's shape at the binding
// edge — one place to adjust if either side moves.
interface ListItem {
  id: string
  content: string
  role: string
  contentType: string
  timestamp: string
  status: string
}

const toListItem = (m: Message): ListItem => ({
  id: m.id,
  content: m.text,
  role: m.role,
  contentType: m.content_type,
  timestamp: m.timestamp,
  status: m.status ?? 'sent',
})

export default controller((ctx: ViewScope<Scope>) => {

    // ── TODO(#SDK-chat): replace the local signal + mock send ──────────
    // The required SDK methods do not exist yet — today `@ship/sdk-client`
    // only exposes `ship.chat.state(args)`. When the gap closes, this
    // controller body becomes:
    //
    //   const messages = query(() =>
    //     ship.chat.messages({ session_id: ctx.sessionId() }))   // live stream
    //   const items = derived(() =>
    //     unwrapOr(messages, { items: [] }).items.map(toListItem))
    //
    //   async function onSubmit(e: Event) {
    //     const { text } = (e as CustomEvent<{ text: string }>).detail
    //     sending.set(true)
    //     try { await ship.chat.send({ session_id: ctx.sessionId(), text }) }
    //     finally { sending.set(false) }
    //     // the live stream above pushes the user + assistant turns back
    //   }
    //
    // `ship.chat.messages` must emit items matching the `Message`
    // contract declared above. Until then, we hold a local array and the
    // submit handler fakes the user turn + an assistant ack so the visual
    // and interaction surface is real and testable now. The component
    // shape (props, slots, emitted events) already matches the real
    // wiring, so only this scope body changes.
    const messages = signal<Message[]>([])
    const sending = signal<boolean>(false)
    const items = derived<ListItem[]>(() => messages().map(toListItem))

    let seq = 0
    const nextId = (): string => `m-${ctx.sessionId() || 'session'}-${seq++}`

    function onSubmit(e: Event): void {
      const detail = (e as CustomEvent<{ text?: string }>).detail
      const text = detail?.text?.trim()
      if (!text) return

      const now = new Date().toISOString()
      const userMsg: Message = {
        id: nextId(),
        role: 'user',
        content_type: 'text',
        text,
        timestamp: now,
        status: 'sent',
      }
      // Mock assistant ack — stands in for the streamed reply the real
      // `ship.chat.send` round-trip will deliver via the live stream.
      const ackMsg: Message = {
        id: nextId(),
        role: 'assistant',
        content_type: 'text',
        text: 'Received. (mock reply — SDK chat stream not wired yet.)',
        timestamp: new Date().toISOString(),
        status: 'sent',
      }
      messages.set([...messages(), userMsg, ackMsg])
    }

    return { messages, sending, items, onSubmit }
})
</controller>

<!--
  <ship-agent-thread> — a complete chat surface bound to a Ship session.

  Composes two `@ship/sail-chat-elements` primitives:
    • <sail-message-list>  keyed, auto-scrolling column of message bubbles
    • <sail-composer>       text input that emits `sui-composer-submit`

  The primitives are unbound (pure `sui-*`, no SDK). This `ship-*`
  wrapper supplies the Ship-runtime binding: it owns the message array
  and the submit handler, turning the framework-tier pieces into a
  working, session-scoped thread.

  Tag prefix `ship-*` marks Ship-runtime dependence. The two composed
  primitives must be registered before this mounts — the host imports
  `@ship/sail-chat-elements` (declared as a dependency of this pack).

  Props
  ─────────────────────────────────────────────────────────────────────
  session-id  (string, required)  Ship session id to thread against.
  ─────────────────────────────────────────────────────────────────────
-->

<!-- <prop name="session-id" type="string" default="" /> -->

<!-- style block removed: use sx / external CSS -->

<column gap="0">
  <sail-message-list items={items()}>
    <span slot="empty">Message the agent to start the conversation.</span>
  </sail-message-list>
  <div>
    <sail-composer
      placeholder="Message agent..."
      disabled={sending()}
      on:sail-composer-submit={onSubmit}
    ></sail-composer>
  </div>
</column>
