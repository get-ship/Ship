import { afterEach, describe, expect, it, vi } from 'vitest'
import '../src/sui-isolation-verify/sui-isolation-verify.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-isolation-verify>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-isolation-verify')).toBeDefined()
  })

  it('renders the loading state before fetch settles', async () => {
    // Never resolve — we capture the loading state.
    vi.spyOn(globalThis, 'fetch').mockImplementation(
      () => new Promise(() => {})
    )
    const { el } = mount(reg, 'sui-isolation-verify')
    await flush()
    expect(shadow(el).querySelector('.loading')?.textContent ?? '').toContain(
      'Running verification'
    )
  })

  it('renders each step from a passing verify response', async () => {
    vi.spyOn(globalThis, 'fetch').mockResolvedValue(
      new Response(
        JSON.stringify({
          ok: true,
          data: {
            ok: true,
            steps: [
              { name: 'capabilities probe', ok: true, detail: 'docker found' },
              { name: 'strategy resolves', ok: true, detail: '' },
            ],
            resolved: null,
          },
        }),
        { status: 200, headers: { 'content-type': 'application/json' } }
      )
    )
    const { el } = mount(reg, 'sui-isolation-verify')
    await new Promise((r) => setTimeout(r, 30))
    await flush()
    const text = shadow(el).textContent ?? ''
    expect(el.getAttribute('ok')).toBe('true')
    expect(text).toContain('capabilities probe')
    expect(text).toContain('strategy resolves')
    expect(text).toContain('docker found')
  })

  it('flags ok=false when any step fails', async () => {
    vi.spyOn(globalThis, 'fetch').mockResolvedValue(
      new Response(
        JSON.stringify({
          ok: true,
          data: {
            ok: false,
            steps: [
              { name: 'capabilities probe', ok: true, detail: '' },
              { name: 'available', ok: false, detail: 'docker daemon not running' },
            ],
          },
        }),
        { status: 200, headers: { 'content-type': 'application/json' } }
      )
    )
    const { el } = mount(reg, 'sui-isolation-verify')
    await new Promise((r) => setTimeout(r, 30))
    await flush()
    expect(el.getAttribute('ok')).toBe('false')
    const text = shadow(el).textContent ?? ''
    expect(text).toContain('docker daemon not running')
    expect(text).toContain('Issues found')
    // Both pass and fail dots should appear.
    expect(shadow(el).querySelector('.dot.pass')).not.toBeNull()
    expect(shadow(el).querySelector('.dot.fail')).not.toBeNull()
  })

  it('marks each step dot with data-pass attribute', async () => {
    vi.spyOn(globalThis, 'fetch').mockResolvedValue(
      new Response(
        JSON.stringify({
          ok: true,
          data: {
            ok: false,
            steps: [
              { name: 'a', ok: true, detail: '' },
              { name: 'b', ok: false, detail: '' },
            ],
          },
        }),
        { status: 200 }
      )
    )
    const { el } = mount(reg, 'sui-isolation-verify')
    await new Promise((r) => setTimeout(r, 30))
    await flush()
    const dots = Array.from(shadow(el).querySelectorAll('.dot'))
    expect(dots.map((d) => d.getAttribute('data-pass'))).toEqual([
      'true',
      'false',
    ])
  })

  it('surfaces fetch failure as an error row', async () => {
    vi.spyOn(globalThis, 'fetch').mockRejectedValue(new Error('network'))
    const { el } = mount(reg, 'sui-isolation-verify')
    await new Promise((r) => setTimeout(r, 30))
    await flush()
    expect(el.getAttribute('ok')).toBe('false')
    const text = shadow(el).textContent ?? ''
    expect(text).toContain('Verify failed')
  })

  it('skips auto-fetch when auto-fetch="false"', async () => {
    vi.restoreAllMocks()
    const spy = vi.spyOn(globalThis, 'fetch')
    mount(reg, 'sui-isolation-verify', { 'auto-fetch': 'false' })
    await new Promise((r) => setTimeout(r, 30))
    await flush()
    expect(spy).not.toHaveBeenCalled()
  })
})
