// @vitest-environment jsdom

import { describe, expect, it, vi } from 'vitest'
import { hydrateOn } from '../../src/hydrate/on'

describe('hydrate/on', () => {
  it('invokes the latest handler returned by the getter', () => {
    const el = document.createElement('button')
    const a = vi.fn()
    const b = vi.fn()
    let handler: () => void = a
    const dispose = hydrateOn(el, 'click', () => handler)
    el.dispatchEvent(new Event('click'))
    expect(a).toHaveBeenCalledTimes(1)
    handler = b
    el.dispatchEvent(new Event('click'))
    expect(a).toHaveBeenCalledTimes(1)
    expect(b).toHaveBeenCalledTimes(1)
    dispose()
    el.dispatchEvent(new Event('click'))
    expect(a).toHaveBeenCalledTimes(1)
    expect(b).toHaveBeenCalledTimes(1)
  })

  it('skips dispatch when the getter does not yield a function', () => {
    const el = document.createElement('button')
    hydrateOn(el, 'click', () => undefined)
    expect(() => el.dispatchEvent(new Event('click'))).not.toThrow()
  })
})
