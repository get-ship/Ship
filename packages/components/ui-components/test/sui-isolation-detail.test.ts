import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import '../src/sui-isolation-detail/sui-isolation-detail.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-isolation-detail>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-isolation-detail')).toBeDefined()
  })

  describe('with explicit props (no auto-fetch)', () => {
    beforeEach(() => {
      vi.spyOn(globalThis, 'fetch').mockResolvedValue(
        new Response(null, { status: 503 })
      )
    })

    it('renders the tier label', async () => {
      const { el } = mount(reg, 'sui-isolation-detail', { tier: 'container' })
      await flush()
      expect(shadow(el).textContent).toContain('Container')
    })

    it('renders strategy when supplied', async () => {
      const { el } = mount(reg, 'sui-isolation-detail', {
        tier: 'container',
        strategy: 'docker',
      })
      await flush()
      expect(shadow(el).textContent).toContain('docker')
    })

    it('renders reason text', async () => {
      const { el } = mount(reg, 'sui-isolation-detail', {
        tier: 'container',
        strategy: 'docker',
        reason: 'user default',
      })
      await flush()
      expect(shadow(el).textContent).toContain('user default')
    })

    it('shows downgraded pill when downgraded=true', async () => {
      const { el } = mount(reg, 'sui-isolation-detail', {
        tier: 'process',
        downgraded: 'true',
        'preferred-tier': 'container',
      })
      await flush()
      const text = shadow(el).textContent ?? ''
      expect(text).toContain('downgraded')
      expect(text).toContain('container')
    })

    it('reflects downgraded=false on the host attribute', async () => {
      // Visual hide is via CSS (`:host([downgraded="false"]) .pill {
      // display: none }`) which JSDOM doesn't compute, so the pill
      // node still exists in shadow DOM. The contract under test
      // here is the host attribute the CSS selector keys off of.
      const { el } = mount(reg, 'sui-isolation-detail', {
        tier: 'container',
        downgraded: 'false',
      })
      await flush()
      expect(el.getAttribute('downgraded')).toBe('false')
    })

    it('renders em-dash for empty strategy', async () => {
      const { el } = mount(reg, 'sui-isolation-detail', { tier: 'none' })
      await flush()
      expect(shadow(el).textContent).toContain('—')
    })

    it('falls back to ellipsis for unknown tier', async () => {
      const { el } = mount(reg, 'sui-isolation-detail', { tier: 'unknown' })
      await flush()
      // The auto-fetch is mocked to 503 -> no decision payload arrives.
      expect(shadow(el).textContent).toContain('…')
    })
  })

  describe('auto-fetch from connect.isolation.status', () => {
    it('populates fields from a successful status response', async () => {
      // Connect wire's `IsolationStatusResponse` is a typed proto —
      // the mock returns the JSON shape Connect actually parses.
      vi.spyOn(globalThis, 'fetch').mockResolvedValue(
        new Response(
          JSON.stringify({
            capabilities: {},
            decision: {
              resolved: {
                strategy: 'docker',
                tier: 'container',
                reason: { case: 'userDefault', value: {} },
                downgraded: false,
              },
            },
          }),
          { status: 200, headers: { 'content-type': 'application/json' } }
        )
      )
      const { el } = mount(reg, 'sui-isolation-detail')
      await new Promise((r) => setTimeout(r, 30))
      await flush()
      expect(el.getAttribute('tier')).toBe('container')
      expect(el.getAttribute('strategy')).toBe('docker')
      expect(shadow(el).textContent).toContain('user default')
    })

    it('surfaces downgrade source from reason.from', async () => {
      vi.spyOn(globalThis, 'fetch').mockResolvedValue(
        new Response(
          JSON.stringify({
            capabilities: {},
            decision: {
              resolved: {
                strategy: 'harness_native',
                tier: 'process',
                reason: { case: 'downgrade', value: { from: 'container', why: '' } },
                downgraded: true,
              },
            },
          }),
          { status: 200, headers: { 'content-type': 'application/json' } }
        )
      )
      const { el } = mount(reg, 'sui-isolation-detail')
      await new Promise((r) => setTimeout(r, 30))
      await flush()
      expect(el.getAttribute('downgraded')).toBe('true')
      expect(el.getAttribute('preferred-tier')).toBe('container')
      expect(shadow(el).textContent).toContain('container')
    })

    it('does NOT auto-fetch when tier is supplied explicitly', async () => {
      vi.restoreAllMocks()
      const spy = vi.spyOn(globalThis, 'fetch')
      const { el } = mount(reg, 'sui-isolation-detail', { tier: 'process' })
      await new Promise((r) => setTimeout(r, 30))
      await flush()
      expect(spy).not.toHaveBeenCalled()
      expect(el.getAttribute('tier')).toBe('process')
    })
  })
})
