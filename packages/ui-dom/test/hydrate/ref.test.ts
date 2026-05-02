// @vitest-environment jsdom

import { describe, expect, it } from 'vitest'
import { signal } from '@ship/ui-runtime'
import { hydrateRef } from '../../src/hydrate/ref'

describe('hydrate/ref', () => {
  it('writes the element to the ref signal and clears on dispose', () => {
    const el = document.createElement('div')
    const ref = signal<Element | null>(null)
    const dispose = hydrateRef(el, ref)
    expect(ref()).toBe(el)
    dispose()
    expect(ref()).toBe(null)
  })
})
