import { afterEach, describe, expect, it } from 'vitest'
import './ship-separator'
import type { ShipSeparator } from './ship-separator'
import { cleanupTestDom } from './util'

describe('ship-separator', () => {
  afterEach(cleanupTestDom)

  it('sets role=separator and aria-orientation', async () => {
    document.body.insertAdjacentHTML('beforeend', '<ship-separator orientation="vertical"></ship-separator>')
    const el = document.body.lastElementChild as ShipSeparator
    await el.updateComplete
    expect(el.getAttribute('role')).toBe('separator')
    expect(el.getAttribute('aria-orientation')).toBe('vertical')
  })
})
