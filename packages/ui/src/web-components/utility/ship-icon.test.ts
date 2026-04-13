import { afterEach, describe, expect, it } from 'vitest'
import './ship-icon'
import type { ShipIcon } from './ship-icon'
import { cleanupTestDom } from '../util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipIcon
  await el.updateComplete
  return el
}

describe('ship-icon', () => {
  afterEach(cleanupTestDom)

  it('renders a named glyph as svg path', async () => {
    const el = await mount('<ship-icon name="check"></ship-icon>')
    const path = el.shadowRoot!.querySelector('path')
    expect(path).not.toBeNull()
    expect(path!.getAttribute('d')).toContain('M20 6')
  })

  it('renders nothing when name is unknown (falls back to slot)', async () => {
    const el = await mount('<ship-icon name="nope"></ship-icon>')
    expect(el.shadowRoot!.querySelector('path')).toBeNull()
  })

  it('reflects size attribute and reacts to change', async () => {
    const el = await mount('<ship-icon name="x" size="sm"></ship-icon>')
    expect(el.getAttribute('size')).toBe('sm')
    el.size = 'lg'
    await el.updateComplete
    expect(el.getAttribute('size')).toBe('lg')
  })

  it('allows raw svg passthrough via default slot', async () => {
    const el = await mount('<ship-icon><svg data-custom></svg></ship-icon>')
    expect(el.querySelector('svg[data-custom]')).not.toBeNull()
  })
})
