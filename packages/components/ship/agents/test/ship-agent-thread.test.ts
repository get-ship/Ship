// <ship-agent-thread> — composition + submit-handler coverage.
//
// The thread composes two `@ship/sail-chat-elements` primitives
// (<sui-message-list> + <sui-composer>) and binds them to a local
// message array. We mount it, fire `sail-composer-submit`, and assert the
// handler appends the user turn (plus the mock assistant ack) so the
// rendered list grows. When the SDK chat stream lands, only the
// controller body changes — this interaction contract stays put.
import { afterEach, describe, expect, it } from 'vitest'
import '@ship/sail-chat-elements'
import '../src/ship-agent-thread/ship-agent-thread.compiled.js'

const containers: HTMLElement[] = []
const mk = (): HTMLElement => {
  const c = document.createElement('div')
  document.body.appendChild(c)
  containers.push(c)
  return c
}
afterEach(() => {
  for (const c of containers.splice(0)) c.remove()
})
const flush = (): Promise<void> => new Promise((r) => setTimeout(r, 0))

type ListEl = HTMLElement & { items: unknown[] }

describe('<ship-agent-thread>', () => {
  it('registers with the global custom-element registry', () => {
    expect(customElements.get('ship-agent-thread')).toBeDefined()
  })

  it('mounts and composes a message-list + composer', async () => {
    const c = mk()
    c.innerHTML = '<ship-agent-thread session-id="s-1"></ship-agent-thread>'
    const host = c.querySelector('ship-agent-thread')!
    await flush()
    const root = host.shadowRoot!
    expect(root.querySelector('sui-message-list')).not.toBeNull()
    expect(root.querySelector('sui-composer')).not.toBeNull()
  })

  it('appends a user turn + mock ack when sui-composer-submit fires', async () => {
    const c = mk()
    c.innerHTML = '<ship-agent-thread session-id="s-1"></ship-agent-thread>'
    const host = c.querySelector('ship-agent-thread')!
    await flush()
    const root = host.shadowRoot!
    const list = root.querySelector('sui-message-list')! as ListEl
    expect(list.items).toEqual([])

    root.querySelector('sui-composer')!.dispatchEvent(
      new CustomEvent('sail-composer-submit', {
        detail: { text: 'hello agent', attachments: [] },
        bubbles: true,
        composed: true,
      }),
    )
    await flush()

    expect(list.items).toHaveLength(2)
    const [user, ack] = list.items as Array<{
      role: string
      content: string
    }>
    expect(user.role).toBe('user')
    expect(user.content).toBe('hello agent')
    expect(ack.role).toBe('assistant')
  })

  it('ignores blank submissions', async () => {
    const c = mk()
    c.innerHTML = '<ship-agent-thread session-id="s-1"></ship-agent-thread>'
    const host = c.querySelector('ship-agent-thread')!
    await flush()
    const root = host.shadowRoot!
    const list = root.querySelector('sui-message-list')! as ListEl

    root.querySelector('sui-composer')!.dispatchEvent(
      new CustomEvent('sail-composer-submit', {
        detail: { text: '   ', attachments: [] },
        bubbles: true,
        composed: true,
      }),
    )
    await flush()
    expect(list.items).toEqual([])
  })
})
