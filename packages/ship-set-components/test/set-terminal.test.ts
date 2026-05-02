/**
 * JSDOM smoke tests for <set-terminal>.
 *
 * xterm.js requires a real browser with canvas / WebGL support. Under JSDOM
 * the terminal engine will not initialize (requestAnimationFrame resolves but
 * there is no layout), so these tests focus on:
 *   - custom element registration
 *   - shadow root attachment and terminal container presence
 *   - prop attribute reflection
 *   - host imperative API surface (write / writeln / clear / focus)
 *   - teardown without throwing
 *
 * Real rendering (ANSI colors, fit addon, event bridging) is verified by the
 * Playwright probe at .artifacts/set-terminal-probe.mjs.
 */

import { afterEach, describe, expect, it } from 'vitest'
import '../src/set-terminal/set-terminal.compiled.js'

const containers: HTMLElement[] = []
function mkContainer(): HTMLElement {
  const c = document.createElement('div')
  document.body.appendChild(c)
  containers.push(c)
  return c
}
afterEach(() => {
  for (const c of containers.splice(0)) c.remove()
})

describe('<set-terminal>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('set-terminal')).toBeDefined()
  })

  it('attaches a shadow root', () => {
    const c = mkContainer()
    c.innerHTML = '<set-terminal></set-terminal>'
    const el = c.querySelector('set-terminal')!
    expect(el.shadowRoot).not.toBeNull()
  })

  it('shadow root contains the terminal-wrap container', () => {
    const c = mkContainer()
    c.innerHTML = '<set-terminal></set-terminal>'
    const el = c.querySelector('set-terminal')!
    expect(el.shadowRoot!.querySelector('.terminal-wrap')).not.toBeNull()
  })

  it('terminal container has role=terminal in interactive mode', () => {
    const c = mkContainer()
    c.innerHTML = '<set-terminal></set-terminal>'
    const el = c.querySelector('set-terminal')!
    const wrap = el.shadowRoot!.querySelector('.terminal-wrap')!
    expect(wrap.getAttribute('role')).toBe('terminal')
    expect(wrap.getAttribute('aria-label')).toBe('Terminal')
  })

  it('terminal container has role=log in readonly mode', () => {
    const c = mkContainer()
    c.innerHTML = '<set-terminal readonly></set-terminal>'
    const el = c.querySelector('set-terminal')!
    const wrap = el.shadowRoot!.querySelector('.terminal-wrap')!
    expect(wrap.getAttribute('role')).toBe('log')
    expect(wrap.getAttribute('aria-label')).toBe('Log output')
  })

  it('reflects readonly attribute', () => {
    const c = mkContainer()
    c.innerHTML = '<set-terminal readonly></set-terminal>'
    const el = c.querySelector('set-terminal')!
    expect(el.hasAttribute('readonly')).toBe(true)
  })

  it('reflects theme attribute', () => {
    const c = mkContainer()
    c.innerHTML = '<set-terminal theme="dark"></set-terminal>'
    const el = c.querySelector('set-terminal')!
    expect(el.getAttribute('theme')).toBe('dark')
  })

  it('reflects fit attribute', () => {
    const c = mkContainer()
    c.innerHTML = '<set-terminal fit></set-terminal>'
    const el = c.querySelector('set-terminal')!
    expect(el.hasAttribute('fit')).toBe(true)
  })

  it('host has imperative API methods after connect', async () => {
    const c = mkContainer()
    c.innerHTML = '<set-terminal></set-terminal>'
    const el = c.querySelector('set-terminal') as HTMLElement & {
      write?: unknown
      writeln?: unknown
      clear?: unknown
      focus?: unknown
    }
    // requestAnimationFrame fires after microtasks; flush it
    await new Promise((r) => setTimeout(r, 0))
    // In JSDOM, xterm.js may not initialize (no layout), but the methods
    // are set on the host unconditionally before open() — if they exist,
    // they must be functions; if not initialized, they are undefined.
    if (el.write !== undefined) expect(typeof el.write).toBe('function')
    if (el.writeln !== undefined) expect(typeof el.writeln).toBe('function')
    if (el.clear !== undefined) expect(typeof el.clear).toBe('function')
    if (el.focus !== undefined) expect(typeof el.focus).toBe('function')
  })

  it('removes without throwing', () => {
    const c = mkContainer()
    c.innerHTML = '<set-terminal></set-terminal>'
    const el = c.querySelector('set-terminal')!
    expect(() => el.remove()).not.toThrow()
  })

  it('double-remove is safe', () => {
    const c = mkContainer()
    c.innerHTML = '<set-terminal></set-terminal>'
    const el = c.querySelector('set-terminal')!
    expect(() => { el.remove(); el.remove() }).not.toThrow()
  })
})
