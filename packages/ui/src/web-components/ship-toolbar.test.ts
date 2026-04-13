import { afterEach, describe, expect, it } from 'vitest'
import './ship-toolbar'
import type { ShipToolbar } from './ship-toolbar'
import { cleanupTestDom } from './util'

describe('ship-toolbar', () => {
  afterEach(cleanupTestDom)

  it('reflects justify/align/gap', async () => {
    document.body.insertAdjacentHTML('beforeend',
      '<ship-toolbar justify="between" align="baseline" gap="lg"></ship-toolbar>')
    const el = document.body.lastElementChild as ShipToolbar
    await el.updateComplete
    expect(el.justify).toBe('between')
    expect(el.getAttribute('gap')).toBe('lg')
  })
})
