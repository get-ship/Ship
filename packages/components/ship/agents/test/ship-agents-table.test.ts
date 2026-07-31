import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { setShipClient } from '@ship/sdk-client'
import '../src/ship-agents-table/ship-agents-table.compiled.js'
import { createContainerRegistry, flush, mount } from '@ship/test-setup'

// The table binds live to ship.agents.summaries(). live() accepts a plain
// value (resolved to `ok`), a rejecting promise (`error`), or a never-settling
// promise (`loading`). A mutable impl lets each test pick the lifecycle state.
let summariesImpl: () => unknown
beforeEach(() => {
  summariesImpl = () => ({ items: [] })
  setShipClient({ agents: { summaries: () => summariesImpl() } } as never)
})

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const table = (el: HTMLElement) =>
  shadow(el).querySelector('sui-table') as (HTMLElement & {
    rows?: unknown[]
    columns?: Array<{ field: string; label: string }>
  }) | null

const SUMMARIES = {
  items: [
    { name: 'code-reviewer', description: 'Reviews PRs', source: 'space-acme', skills_count: 3, mcp_count: 1 },
    { name: 'lead-tracker', description: 'Tracks leads', source: 'space-acme', skills_count: 0, mcp_count: 2 },
  ],
}

describe('<ship-agents-table>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-agents-table')).toBeDefined()
  })

  it('renders a sui-table once the live summaries resolve', async () => {
    summariesImpl = () => SUMMARIES
    const { el } = mount(reg, 'ship-agents-table')
    await flush()
    expect(table(el)).not.toBeNull()
  })

  it('passes the summaries items through as the table rows prop', async () => {
    summariesImpl = () => SUMMARIES
    const { el } = mount(reg, 'ship-agents-table')
    await flush()
    const rows = table(el)?.rows
    expect(rows?.length).toBe(2)
    expect((rows?.[0] as { name: string }).name).toBe('code-reviewer')
    expect((rows?.[1] as { name: string }).name).toBe('lead-tracker')
  })

  it('binds the default five-column definition (field + label)', async () => {
    summariesImpl = () => SUMMARIES
    const { el } = mount(reg, 'ship-agents-table')
    await flush()
    const cols = table(el)?.columns
    expect(cols?.map((c) => c.field)).toEqual([
      'name', 'description', 'source', 'skills_count', 'mcp_count',
    ])
    expect(cols?.map((c) => c.label)).toEqual([
      'Name', 'Description', 'Source', 'Skills', 'MCP',
    ])
  })

  it('renders an empty rows set when there are no agents', async () => {
    summariesImpl = () => ({ items: [] })
    const { el } = mount(reg, 'ship-agents-table')
    await flush()
    expect(table(el)?.rows).toEqual([])
  })

  it('shows the skeleton loading boundary while summaries are pending', async () => {
    summariesImpl = () => new Promise(() => {}) // never settles
    const { el } = mount(reg, 'ship-agents-table')
    await flush()
    expect(shadow(el).querySelector('sui-skeleton')).not.toBeNull()
    expect(table(el)).toBeNull()
  })

  it('shows the error boundary when summaries reject', async () => {
    summariesImpl = () => Promise.reject(new Error('boom'))
    const { el } = mount(reg, 'ship-agents-table')
    await flush()
    await flush()
    const errState = shadow(el).querySelector('sui-error-state')
    expect(errState).not.toBeNull()
    expect(errState?.textContent).toContain('boom')
    expect(table(el)).toBeNull()
  })
})
