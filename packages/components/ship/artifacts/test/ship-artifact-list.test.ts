import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { setShipClient } from '@ship/sdk-client'
import '../src/ship-artifact-list/ship-artifact-list.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

// The list binds live to ship.artifacts.list({}). live() accepts a plain
// value (resolved to `ok`), a rejecting promise (`error`), or a
// never-settling promise (`loading`). A mutable impl lets each test pick
// the lifecycle state — mirrors the ship-agents-table test pattern.
let listImpl: () => unknown
beforeEach(() => {
  listImpl = () => ({ items: [] })
  setShipClient({ artifacts: { list: () => listImpl() } } as never)
})

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const table = (el: HTMLElement) =>
  shadow(el).querySelector('sui-table') as (HTMLElement & {
    rows?: unknown[]
    columns?: Array<{ field: string; label: string }>
  }) | null

const SAMPLE = {
  items: [
    { id: 'a1', name: 'arch.png', mime: 'image/png', size: 100, owner_actor: 'claude', created_at: '2026-05-31T20:10:00Z' },
    { id: 'a2', name: 'notes.md', mime: 'text/markdown', size: 200, owner_actor: null, created_at: '2026-05-30T11:00:00Z' },
  ],
}

describe('<ship-artifact-list>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-artifact-list')).toBeDefined()
  })

  it('renders a sui-table once the live list resolves', async () => {
    listImpl = () => SAMPLE
    const { el } = mount(reg, 'ship-artifact-list')
    await flush()
    expect(table(el)).not.toBeNull()
  })

  it('passes the list items through as the table rows prop', async () => {
    listImpl = () => SAMPLE
    const { el } = mount(reg, 'ship-artifact-list')
    await flush()
    const rows = table(el)?.rows
    expect(rows?.length).toBe(2)
    expect((rows?.[0] as { name: string }).name).toBe('arch.png')
  })

  it('binds the default six-column definition (field + label)', async () => {
    listImpl = () => SAMPLE
    const { el } = mount(reg, 'ship-artifact-list')
    await flush()
    const cols = table(el)?.columns
    expect(cols?.map((c) => c.field)).toEqual([
      'name', 'mime', 'viewer_label', 'owner_actor', 'created_short', 'size_label',
    ])
    expect(cols?.map((c) => c.label)).toEqual([
      'Name', 'Type', 'Viewer', 'Actor', 'Age', 'Size',
    ])
  })

  it('shows the skeleton loading boundary while the list is pending', async () => {
    listImpl = () => new Promise(() => {}) // never settles
    const { el } = mount(reg, 'ship-artifact-list')
    await flush()
    expect(shadow(el).querySelector('sui-skeleton')).not.toBeNull()
    expect(table(el)).toBeNull()
  })

  it('shows the error boundary when the list rejects', async () => {
    listImpl = () => Promise.reject(new Error('boom'))
    const { el } = mount(reg, 'ship-artifact-list')
    await flush()
    await flush()
    const errState = shadow(el).querySelector('sui-error-state')
    expect(errState).not.toBeNull()
    expect(errState?.textContent).toContain('boom')
    expect(table(el)).toBeNull()
  })
})
