// Tests for <ship-agent-detail> — the hub /studio/agents/:id viewer.
//
// The detail binds live to ship.agents.detail({ id }). live() accepts
// a plain value (resolved to `ok`), a rejecting promise (`error`),
// or a never-settling promise (`loading`). A mutable impl lets each
// test pick the lifecycle state.
//
// Coverage:
//   1. Pure helpers (pickHue, pickInitials, isHue) — exported from
//      the controller as __test__.
//   2. Custom element registration.
//   3. Loading / error / not-found / data state branches.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { setShipClient } from '@ship/sdk-client'
import '../src/ship-agent-detail/ship-agent-detail.compiled.js'
import { createContainerRegistry, flush, mount } from '@ship/test-setup'
import { __test__ } from '../src/ship-agent-detail/ship-agent-detail.controller'

// The detail binds live to ship.agents.detail({ id }). Stub the SDK
// with a mutable impl so each test can pick the lifecycle state.
let detailImpl: () => unknown
beforeEach(() => {
  detailImpl = () => ({
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
  })
  setShipClient({ agents: { detail: () => detailImpl() } } as never)
})

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const q = (el: HTMLElement, sel: string) =>
  shadow(el).querySelector(`[data-test="${sel}"]`)

// ── Pure helpers ───────────────────────────────────────────────────────

describe('ship-agent-detail helpers', () => {
  it('pickHue: slate for empty id', () => {
    expect(__test__.pickHue('')).toBe('slate')
  })

  it('pickHue: deterministic per id', () => {
    expect(__test__.pickHue('rust-compiler')).toBe(__test__.pickHue('rust-compiler'))
  })

  it('pickHue: returns a valid hue name', () => {
    const palette = new Set<string>(__test__.HUES)
    for (const id of ['a', 'b', 'rust-compiler', 'etl-runner']) {
      expect(palette.has(__test__.pickHue(id))).toBe(true)
    }
  })

  it('pickInitials: two letters of hyphenated id', () => {
    expect(__test__.pickInitials('rust-compiler')).toBe('RC')
    expect(__test__.pickInitials('test_runner')).toBe('TR')
    expect(__test__.pickInitials('agent zero')).toBe('AZ')
  })

  it('pickInitials: two letters of single-token id', () => {
    expect(__test__.pickInitials('shipwright')).toBe('SH')
    expect(__test__.pickInitials('gatekeeper')).toBe('GA')
  })

  it('pickInitials: ? for empty', () => {
    expect(__test__.pickInitials('')).toBe('?')
  })

  it('isHue: validates against the palette', () => {
    expect(__test__.isHue('orange')).toBe(true)
    expect(__test__.isHue('slate')).toBe(true)
    expect(__test__.isHue('fuchsia')).toBe(false) // not in the palette
    expect(__test__.isHue('nope')).toBe(false)
  })
})

// ── Component ──────────────────────────────────────────────────────────

describe('<ship-agent-detail>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-agent-detail')).toBeDefined()
  })

  it('renders the loading skeleton while detail is pending', async () => {
    detailImpl = () => new Promise(() => {}) // never settles
    const { el } = mount(reg, 'ship-agent-detail', { id: 'lead-tracker' })
    await flush()
    expect(q(el, 'agents-detail-loading')).not.toBeNull()
  })

  it('renders the error boundary when detail rejects', async () => {
    detailImpl = () => Promise.reject(new Error('boom'))
    const { el } = mount(reg, 'ship-agent-detail', { id: 'lead-tracker' })
    await flush()
    await flush()
    const errState = q(el, 'agents-detail-error')
    expect(errState).not.toBeNull()
    expect(errState?.textContent).toContain('boom')
  })

  it('renders the not-found empty state when detail returns a falsy payload', async () => {
    detailImpl = () => null
    const { el } = mount(reg, 'ship-agent-detail', { id: 'ghost' })
    await flush()
    expect(q(el, 'agents-detail-not-found')).not.toBeNull()
  })

  it('renders the agent name + id + description from the detail payload', async () => {
    const { el } = mount(reg, 'ship-agent-detail', { id: 'lead-tracker' })
    await flush()
    expect(q(el, 'agents-detail-name')?.textContent).toBe('lead-tracker')
    expect(q(el, 'agents-detail-id')?.textContent).toBe('lead-tracker')
    expect(q(el, 'agents-detail-description')?.textContent).toBe('Tracks inbound leads.')
  })

  it('renders the harness, model, and status', async () => {
    const { el } = mount(reg, 'ship-agent-detail', { id: 'lead-tracker' })
    await flush()
    expect(q(el, 'agents-detail-harness')?.textContent).toBe('floki')
    expect(q(el, 'agents-detail-model')?.textContent).toBe('opus-4-8')
    expect(q(el, 'agents-detail-status-label')?.textContent).toBe('idle')
  })

  it('renders capability chips', async () => {
    const { el } = mount(reg, 'ship-agent-detail', { id: 'lead-tracker' })
    await flush()
    const capsRoot = q(el, 'agents-detail-capabilities')
    const chips = capsRoot?.querySelectorAll('sui-chip')
    expect(chips?.length).toBe(2)
    const text = capsRoot?.textContent ?? ''
    expect(text).toContain('collections')
    expect(text).toContain('C2')
  })

  it('emits ship-agent-start with the agent id on Start click', async () => {
    const { el } = mount(reg, 'ship-agent-detail', { id: 'lead-tracker' })
    await flush()
    const onStart = vi.fn()
    el.addEventListener('ship-agent-start', onStart)
    ;(q(el, 'agents-detail-start') as HTMLElement).click()
    expect(onStart).toHaveBeenCalledTimes(1)
    expect((onStart.mock.calls[0][0] as CustomEvent).detail).toEqual({ id: 'lead-tracker' })
  })

  it('emits ship-agent-back on the back link click', async () => {
    const { el } = mount(reg, 'ship-agent-detail', { id: 'lead-tracker' })
    await flush()
    const onBack = vi.fn()
    el.addEventListener('ship-agent-back', onBack)
    ;(q(el, 'agents-detail-back') as HTMLElement).click()
    expect(onBack).toHaveBeenCalledTimes(1)
  })

  it('binds the hue tile color from the data-driven hue name', async () => {
    const { el } = mount(reg, 'ship-agent-detail', { id: 'lead-tracker' })
    await flush()
    const hue = q(el, 'agents-detail-hue') as HTMLElement
    const style = hue.getAttribute('style') ?? ''
    // The controller-derived hue for 'lead-tracker' is deterministic;
    // the bound style is `style="--tile-color: var(--hue-{name})"`.
    expect(style).toMatch(/^--tile-color: var\(--hue-/)
  })
})
