// Tests for <ship-agent-list> — the hub /studio/agents index composite.
//
// The list binds live to ship.agents.summaries(). live() accepts a
// plain value (resolved to `ok`), a rejecting promise (`error`), or
// a never-settling promise (`loading`). A mutable impl lets each
// test pick the lifecycle state.
//
// Coverage:
//   1. Pure helpers (pickHue, pickLetter, readRows, matchesKind,
//      matchesQuery) — exported from the controller as __test__.
//   2. Custom element registration.
//   3. Loading / error / empty / data state branches.

import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { setShipClient } from '@ship/sdk-client'
import '../src/ship-agent-list/ship-agent-list.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'
import { __test__, readRows } from '../src/ship-agent-list/ship-agent-list.controller'

// The list binds live to ship.agents.summaries(). Stub the SDK with
// a mutable impl — live() accepts a plain value and resolves it to
// `ok`; a rejecting promise resolves to `error`; a never-settling
// promise leaves the signal in `loading`.
let summariesImpl: () => unknown
beforeEach(() => {
  summariesImpl = () => ({ items: [] })
  setShipClient({ agents: { summaries: () => summariesImpl() } } as never)
})

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const q = (el: HTMLElement, sel: string) =>
  shadow(el).querySelector(`[data-test="${sel}"]`)

const SUMMARIES = {
  items: [
    {
      id: 'rust-compiler',
      name: 'Rust Compiler',
      description: 'Compiles Rust to WASM',
      source: 'PROJECT AGENTS',
      active: true,
      tags: ['rust', 'wasm'],
      skills_count: 3,
      mcp_count: 2,
      rules_count: 1,
    },
    {
      id: 'doc-reviewer',
      name: 'Doc Reviewer',
      description: 'Reviews doc PRs',
      source: 'USER AGENTS',
      active: false,
      tags: ['docs'],
      skills_count: 1,
      mcp_count: 0,
      rules_count: 0,
    },
  ],
}

// ── Pure helpers ───────────────────────────────────────────────────────

describe('ship-agent-list helpers', () => {
  it('pickHue: slate for empty id', () => {
    expect(__test__.pickHue('')).toBe('slate')
  })

  it('pickHue: deterministic per id', () => {
    expect(__test__.pickHue('rust-compiler')).toBe(__test__.pickHue('rust-compiler'))
  })

  it('pickHue: returns a valid hue name', () => {
    const palette = new Set<string>(__test__.HUES)
    for (const id of ['rust-compiler', 'doc-reviewer', 'etl-runner', 'a', 'zzz']) {
      expect(palette.has(__test__.pickHue(id))).toBe(true)
    }
  })

  it('pickHue: different ids generally pick different hues', () => {
    const hues = new Set([
      __test__.pickHue('one'),
      __test__.pickHue('two'),
      __test__.pickHue('three'),
      __test__.pickHue('four'),
      __test__.pickHue('five'),
    ])
    expect(hues.size).toBeGreaterThan(1)
  })

  it('pickLetter: two letters of hyphenated id', () => {
    expect(__test__.pickLetter('rust-compiler', '')).toBe('RC')
    expect(__test__.pickLetter('test_runner', '')).toBe('TR')
  })

  it('pickLetter: ? for empty', () => {
    expect(__test__.pickLetter('', '')).toBe('?')
  })

  it('readRows: defensive on partial payload', () => {
    const rows = readRows({ items: [{ id: 'a', name: 'A' }] })
    expect(rows.length).toBe(1)
    expect(rows[0].id).toBe('a')
    expect(rows[0].name).toBe('A')
    expect(rows[0].tags).toEqual([])
    expect(rows[0].skills_count).toBe(0)
  })

  it('readRows: skips items without a string id', () => {
    const rows = readRows({ items: [{ id: 123 }, { id: 'b' }, null] })
    expect(rows.length).toBe(1)
    expect(rows[0].id).toBe('b')
  })

  it('readRows: handles empty / missing items', () => {
    expect(readRows({})).toEqual([])
    expect(readRows({ items: [] })).toEqual([])
    expect(readRows(null)).toEqual([])
  })

  it('matchesKind: all matches everything', () => {
    const a = { source: 'PROJECT AGENTS' } as never
    expect(__test__.matchesKind(a, 'all')).toBe(true)
  })

  it('matchesKind: project matches PROJECT AGENTS only', () => {
    expect(__test__.matchesKind({ source: 'PROJECT AGENTS' } as never, 'project')).toBe(true)
    expect(__test__.matchesKind({ source: 'USER AGENTS' } as never, 'project')).toBe(false)
  })

  it('matchesKind: user matches USER AGENTS only', () => {
    expect(__test__.matchesKind({ source: 'USER AGENTS' } as never, 'user')).toBe(true)
    expect(__test__.matchesKind({ source: 'PROJECT AGENTS' } as never, 'user')).toBe(false)
  })

  it('matchesQuery: matches name / id / description / tags', () => {
    const a = { id: 'a', name: 'rust compiler', description: 'wasm tool', tags: ['compile'] } as never
    expect(__test__.matchesQuery(a, 'rust')).toBe(true)
    expect(__test__.matchesQuery(a, 'WASM')).toBe(true)
    expect(__test__.matchesQuery(a, 'compile')).toBe(true)
    expect(__test__.matchesQuery(a, 'xyz')).toBe(false)
  })

  it('matchesQuery: empty query matches everything', () => {
    const a = { id: 'a', name: 'a', description: '', tags: [] } as never
    expect(__test__.matchesQuery(a, '')).toBe(true)
  })
})

// ── Component ──────────────────────────────────────────────────────────

describe('<ship-agent-list>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-agent-list')).toBeDefined()
  })

  it('renders the loading skeleton while summaries are pending', async () => {
    summariesImpl = () => new Promise(() => {}) // never settles
    const { el } = mount(reg, 'ship-agent-list')
    await flush()
    expect(q(el, 'agents-list-loading')).not.toBeNull()
  })

  it('renders the error boundary when summaries reject', async () => {
    summariesImpl = () => Promise.reject(new Error('boom'))
    const { el } = mount(reg, 'ship-agent-list')
    await flush()
    await flush()
    const errState = q(el, 'agents-list-error')
    expect(errState).not.toBeNull()
    expect(errState?.textContent).toContain('boom')
  })

  it('renders the empty-state when summaries return zero items', async () => {
    summariesImpl = () => ({ items: [] })
    const { el } = mount(reg, 'ship-agent-list')
    await flush()
    expect(q(el, 'agents-list-empty')).not.toBeNull()
  })

  it('renders one grid cell per agent on data', async () => {
    summariesImpl = () => SUMMARIES
    const { el } = mount(reg, 'ship-agent-list')
    await flush()
    const cells = shadow(el).querySelectorAll('[data-test="agents-list-cell"]')
    expect(cells.length).toBe(2)
  })

  it('renders the active chip on the active agent', async () => {
    summariesImpl = () => SUMMARIES
    const { el } = mount(reg, 'ship-agent-list')
    await flush()
    // rust-compiler is the only active agent; the active chip
    // renders for that one. The other agent's cell has no chip.
    const chips = shadow(el).querySelectorAll('[data-test="agents-list-active"]')
    expect(chips.length).toBe(1)
    const activeCard = chips[0]?.closest('[data-test="agents-list-cell"]')
    expect(activeCard?.getAttribute('data-id')).toBe('rust-compiler')
  })

  it('header sub reflects the agent count', async () => {
    summariesImpl = () => SUMMARIES
    const { el } = mount(reg, 'ship-agent-list')
    await flush()
    expect(q(el, 'agents-list-sub')?.textContent).toBe('2 agents')
  })
})
