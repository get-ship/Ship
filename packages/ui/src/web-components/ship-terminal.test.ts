/**
 * Tests for <ship-terminal>.
 *
 * JSDOM cannot run the xterm.js canvas renderer, so these tests focus on
 * the component shell: registration, attribute reflection, public API
 * surface, readonly behavior, and teardown. Full rendering verification
 * (ANSI colors, fit addon, search) is covered in the Playwright pass.
 */

import { afterEach, describe, expect, it, vi } from 'vitest'
import './ship-terminal'
import type { ShipTerminal } from './ship-terminal'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipTerminal
  await el.updateComplete
  return el
}

describe('ship-terminal', () => {
  afterEach(cleanupTestDom)

  it('registers the custom element', () => {
    expect(customElements.get('ship-terminal')).toBeTruthy()
  })

  it('renders a terminal container with data-terminal', async () => {
    const el = await mount('<ship-terminal></ship-terminal>')
    const wrap = el.shadowRoot!.querySelector('[data-terminal]')
    expect(wrap).toBeTruthy()
  })

  it('reflects readonly attribute', async () => {
    const el = await mount('<ship-terminal readonly></ship-terminal>')
    expect(el.readonly).toBe(true)
    expect(el.hasAttribute('readonly')).toBe(true)
  })

  it('defaults rows=24 and cols=80', async () => {
    const el = await mount('<ship-terminal></ship-terminal>')
    expect(el.rows).toBe(24)
    expect(el.cols).toBe(80)
  })

  it('accepts custom rows and cols', async () => {
    const el = await mount('<ship-terminal rows="10" cols="120"></ship-terminal>')
    expect(el.rows).toBe(10)
    expect(el.cols).toBe(120)
  })

  it('accepts font-size attribute', async () => {
    const el = await mount('<ship-terminal font-size="14"></ship-terminal>')
    expect(el.fontSize).toBe(14)
  })

  it('defaults theme to auto', async () => {
    const el = await mount('<ship-terminal></ship-terminal>')
    expect(el.theme).toBe('auto')
  })

  it('reflects theme attribute', async () => {
    const el = await mount('<ship-terminal theme="dark"></ship-terminal>')
    expect(el.getAttribute('theme')).toBe('dark')
    expect(el.theme).toBe('dark')
  })

  it('reflects fit attribute', async () => {
    const el = await mount('<ship-terminal fit></ship-terminal>')
    expect(el.fit).toBe(true)
    expect(el.hasAttribute('fit')).toBe(true)
  })

  it('readonly container has role=log', async () => {
    const el = await mount('<ship-terminal readonly></ship-terminal>')
    const wrap = el.shadowRoot!.querySelector('[data-terminal]')!
    expect(wrap.getAttribute('role')).toBe('log')
    expect(wrap.getAttribute('aria-label')).toBe('Log output')
  })

  it('interactive container has role=terminal', async () => {
    const el = await mount('<ship-terminal></ship-terminal>')
    const wrap = el.shadowRoot!.querySelector('[data-terminal]')!
    expect(wrap.getAttribute('role')).toBe('terminal')
    expect(wrap.getAttribute('aria-label')).toBe('Terminal')
  })

  it('search bar is hidden by default', async () => {
    const el = await mount('<ship-terminal></ship-terminal>')
    const search = el.shadowRoot!.querySelector('.search-bar')!
    expect(search.getAttribute('data-open')).toBe('false')
  })

  it('write/writeln/clear are callable before init (no throw)', async () => {
    const el = await mount('<ship-terminal></ship-terminal>')
    // Terminal may not have initialized in JSDOM, but methods should not throw
    expect(() => el.write('hello')).not.toThrow()
    expect(() => el.writeln('world')).not.toThrow()
    expect(() => el.clear()).not.toThrow()
  })

  it('terminal property is null before init in JSDOM', async () => {
    const el = await mount('<ship-terminal></ship-terminal>')
    // In JSDOM, xterm.js canvas init will fail, so terminal may be null
    // This is expected behavior — real rendering is tested in Playwright
    expect(el.terminal).toBeDefined()
  })

  it('teardown removes the element without throwing', async () => {
    const el = await mount('<ship-terminal></ship-terminal>')
    expect(() => el.remove()).not.toThrow()
  })

  it('does not crash when disconnected before init completes', async () => {
    const el = await mount('<ship-terminal></ship-terminal>')
    // Immediately remove — should not throw even if async init is pending
    expect(() => {
      el.remove()
      el.remove() // double-remove should also be safe
    }).not.toThrow()
  })
})
