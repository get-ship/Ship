// Behavioral spec for <ship-agent-chat>, the conversation shell. The shell
// is presentation + event-out: it never touches the SDK, just emits the
// composer's submit, the cancel (Escape), and the switch affordance. The
// host slots each turn and is the one that calls ship.agents.send() etc.

import { afterEach, describe, expect, it, vi } from 'vitest'
import '../src/ship-agent-chat/ship-agent-chat.compiled.js'
import { createContainerRegistry, flush, mount } from '@ship/test-setup'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot ?? el
const q = (el: HTMLElement, sel: string) =>
  shadow(el).querySelector(`[data-test="${sel}"]`)
const qInput = (el: HTMLElement) =>
  shadow(el).querySelector<HTMLInputElement>('[data-test="composer-input"]')

describe('<ship-agent-chat>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-agent-chat')).toBeDefined()
  })

  it('renders agent + session in the header and reflects surface to the host attribute', async () => {
    const { el } = mount(reg, 'ship-agent-chat', {
      agent: 'Editor',
      session: 'sess-42',
      surface: 'drawer',
    })
    await flush()
    expect(q(el, 'agent-name')?.textContent).toBe('Editor')
    expect(q(el, 'session-id')?.textContent).toBe('sess-42')
    expect(el.getAttribute('surface')).toBe('drawer')
  })

  it('reflects the state enum (idle | streaming | awaiting)', async () => {
    for (const state of ['idle', 'streaming', 'awaiting'] as const) {
      const { el } = mount(reg, 'ship-agent-chat', { agent: 'a', session: 's', state })
      await flush()
      expect(el.getAttribute('state')).toBe(state)
    }
  })

  it('emits ship-agent-chat-send { text, agent, session } on send click and clears the composer', async () => {
    const { el } = mount(reg, 'ship-agent-chat', { agent: 'Editor', session: 'sess-42' })
    await flush()
    const input = qInput(el)!
    input.value = 'draft the launch post'
    const onSend = vi.fn()
    el.addEventListener('ship-agent-chat-send', onSend)

    q(el, 'composer-send')?.dispatchEvent(new Event('click', { bubbles: true }))
    await flush()

    expect(onSend).toHaveBeenCalledTimes(1)
    expect((onSend.mock.calls[0][0] as CustomEvent).detail).toEqual({
      text: 'draft the launch post',
      agent: 'Editor',
      session: 'sess-42',
    })
    // Composer cleared so the next turn starts empty.
    expect(input.value).toBe('')
  })

  it('emits ship-agent-chat-send on Enter (no Shift)', async () => {
    const { el } = mount(reg, 'ship-agent-chat', { agent: 'a', session: 's' })
    await flush()
    const input = qInput(el)!
    input.value = 'go'
    const onSend = vi.fn()
    el.addEventListener('ship-agent-chat-send', onSend)

    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    await flush()
    expect(onSend).toHaveBeenCalledTimes(1)
    expect((onSend.mock.calls[0][0] as CustomEvent).detail.text).toBe('go')
  })

  it('does NOT emit on Shift+Enter (newline, not submit)', async () => {
    const { el } = mount(reg, 'ship-agent-chat', { agent: 'a', session: 's' })
    await flush()
    const input = qInput(el)!
    input.value = 'go'
    const onSend = vi.fn()
    el.addEventListener('ship-agent-chat-send', onSend)

    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', shiftKey: true, bubbles: true }))
    await flush()
    expect(onSend).not.toHaveBeenCalled()
    expect(input.value).toBe('go') // not cleared
  })

  it('does not submit on empty / whitespace-only input', async () => {
    const { el } = mount(reg, 'ship-agent-chat', { agent: 'a', session: 's' })
    await flush()
    const onSend = vi.fn()
    el.addEventListener('ship-agent-chat-send', onSend)

    const input = qInput(el)!
    input.value = '   '
    q(el, 'composer-send')?.dispatchEvent(new Event('click', { bubbles: true }))
    await flush()
    expect(onSend).not.toHaveBeenCalled()
  })

  it('emits ship-agent-chat-cancel { agent, session } on Escape', async () => {
    const { el } = mount(reg, 'ship-agent-chat', { agent: 'Editor', session: 'sess-7' })
    await flush()
    const onCancel = vi.fn()
    el.addEventListener('ship-agent-chat-cancel', onCancel)

    qInput(el)!.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    await flush()
    expect(onCancel).toHaveBeenCalledTimes(1)
    expect((onCancel.mock.calls[0][0] as CustomEvent).detail).toEqual({
      agent: 'Editor',
      session: 'sess-7',
    })
  })

  it('emits ship-agent-chat-switch { agent, session } on the Switch affordance', async () => {
    const { el } = mount(reg, 'ship-agent-chat', { agent: 'Editor', session: 'sess-7' })
    await flush()
    const onSwitch = vi.fn()
    el.addEventListener('ship-agent-chat-switch', onSwitch)

    q(el, 'switch')?.dispatchEvent(new Event('click', { bubbles: true }))
    await flush()
    expect(onSwitch).toHaveBeenCalledTimes(1)
    expect((onSwitch.mock.calls[0][0] as CustomEvent).detail).toEqual({
      agent: 'Editor',
      session: 'sess-7',
    })
  })
})
