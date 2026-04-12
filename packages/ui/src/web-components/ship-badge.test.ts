import { afterEach, describe, expect, it } from 'vitest'
import './ship-badge'
import type { ShipBadge } from './ship-badge'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipBadge
  await el.updateComplete
  return el
}

describe('ship-badge', () => {
  afterEach(cleanupTestDom)

  it('defaults to tone=muted', async () => {
    const el = await mount('<ship-badge>ok</ship-badge>')
    expect(el.tone).toBe('muted')
    expect(el.getAttribute('tone')).toBe('muted')
  })

  it('reflects tone attribute', async () => {
    const el = await mount('<ship-badge tone="success">done</ship-badge>')
    expect(el.tone).toBe('success')
  })

  it('slots label text', async () => {
    const el = await mount('<ship-badge>completed</ship-badge>')
    expect(el.textContent).toBe('completed')
  })
})
