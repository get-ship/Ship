import { afterEach, describe, expect, it } from 'vitest'
import './ship-timestamp'
import { ShipTimestamp, formatRelative } from './ship-timestamp'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipTimestamp
  await el.updateComplete
  return el
}

describe('ship-timestamp', () => {
  afterEach(cleanupTestDom)

  it('formats relative time', () => {
    const base = new Date('2024-01-01T12:00:00Z').getTime()
    expect(formatRelative(new Date(base - 30_000).toISOString(), base)).toBe('just now')
    expect(formatRelative(new Date(base - 120_000).toISOString(), base)).toBe('2m ago')
    expect(formatRelative(new Date(base - 2 * 3_600_000).toISOString(), base)).toBe('2h ago')
  })

  it('sets title to absolute ISO', async () => {
    const iso = new Date(Date.now() - 60_000).toISOString()
    const el = await mount(`<ship-timestamp value="${iso}"></ship-timestamp>`)
    expect(el.title).toBe(iso)
  })

  it('renders absolute when format=absolute', async () => {
    const iso = new Date(Date.now() - 60_000).toISOString()
    const el = await mount(`<ship-timestamp value="${iso}" format="absolute"></ship-timestamp>`)
    expect(el.shadowRoot!.textContent).toContain(iso)
  })
})
