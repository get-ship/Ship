import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest'
import { setShipClient } from '@ship/sdk-client'
import '../src/ship-agent-card/ship-agent-card.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

// The card binds live to ship.agents.detail({ id }). Stub the SDK with a
// static detail object — live() accepts a plain value and resolves it to the
// `ok` state — so the elevated identity surface renders without a daemon.
const DETAIL = {
  id: 'lead-tracker',
  name: 'lead-tracker',
  description: 'Tracks inbound leads.',
  source: 'space-acme',
  tags: ['inbox'],
  harness: 'floki',
  model: 'opus-4-8',
  status: 'idle',
  capabilities: [
    { label: 'collections', tier: 'C2' },
    { label: 'mcp', tier: 'C1' },
  ],
}

beforeAll(() => {
  setShipClient({ agents: { detail: () => DETAIL } } as never)
})

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const q = (el: HTMLElement, sel: string) =>
  shadow(el).querySelector(`[data-test="${sel}"]`)

describe('<ship-agent-card>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-agent-card')).toBeDefined()
  })

  it('keeps the live binding: renders name + source from ship.agents.detail()', async () => {
    const { el } = mount(reg, 'ship-agent-card', { id: 'lead-tracker' })
    await flush()
    expect(q(el, 'agent-name')?.textContent).toBe('lead-tracker')
    expect(q(el, 'agent-source')?.textContent).toBe('space-acme')
  })

  it('renders the harness badge, model and status dot', async () => {
    const { el } = mount(reg, 'ship-agent-card', { id: 'lead-tracker' })
    await flush()
    expect(q(el, 'agent-harness')?.textContent).toBe('floki')
    expect(q(el, 'agent-model')?.textContent).toBe('opus-4-8')
    expect(q(el, 'agent-status')?.textContent).toContain('idle')
    expect(shadow(el).querySelector('[data-test="agent-status"] .sdot')).not.toBeNull()
  })

  it('renders capability tier chips', async () => {
    const { el } = mount(reg, 'ship-agent-card', { id: 'lead-tracker' })
    await flush()
    const chips = q(el, 'agent-caps')?.querySelectorAll('sui-chip')
    expect(chips?.length).toBe(2)
    const capsText = q(el, 'agent-caps')?.textContent ?? ''
    expect(capsText).toContain('collections')
    expect(capsText).toContain('C2')
    // C2 → warning tone, C1 → info tone (blast-tier → chip-tone map).
    expect(chips?.[0].getAttribute('tone')).toBe('warning')
    expect(chips?.[1].getAttribute('tone')).toBe('info')
  })

  it('emits ship-start with the agent id on Start click', async () => {
    const { el } = mount(reg, 'ship-agent-card', { id: 'lead-tracker' })
    await flush()
    const onStart = vi.fn()
    el.addEventListener('ship-start', onStart)
    ;(q(el, 'agent-start') as HTMLElement).click()
    expect(onStart).toHaveBeenCalledTimes(1)
    expect((onStart.mock.calls[0][0] as CustomEvent).detail).toEqual({ id: 'lead-tracker' })
  })
})
