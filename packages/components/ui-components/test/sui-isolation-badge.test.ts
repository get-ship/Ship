import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import '../src/sui-isolation-badge/sui-isolation-badge.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-isolation-badge>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-isolation-badge')).toBeDefined()
  })

  describe('with explicit tier (no auto-fetch)', () => {
    beforeEach(() => {
      // Block /api/v1/isolation/status fetch so these stay pure.
      vi.spyOn(globalThis, 'fetch').mockResolvedValue(
        new Response(null, { status: 503 })
      )
    })

    it('renders the Not sandboxed label for tier=none', async () => {
      const { el } = mount(reg, 'sui-isolation-badge', { tier: 'none' })
      await flush()
      const label = shadow(el).querySelector('.label')?.textContent?.trim()
      expect(label).toBe('Not sandboxed')
    })

    it('renders Container for tier=container', async () => {
      const { el } = mount(reg, 'sui-isolation-badge', { tier: 'container' })
      await flush()
      expect(shadow(el).querySelector('.label')?.textContent?.trim()).toBe('Container')
    })

    it('renders Sandboxed for tier=process', async () => {
      const { el } = mount(reg, 'sui-isolation-badge', { tier: 'process' })
      await flush()
      expect(shadow(el).querySelector('.label')?.textContent?.trim()).toBe('Sandboxed')
    })

    it('renders Cloud for tier=cloud', async () => {
      const { el } = mount(reg, 'sui-isolation-badge', { tier: 'cloud' })
      await flush()
      expect(shadow(el).querySelector('.label')?.textContent?.trim()).toBe('Cloud')
    })

    it('label prop overrides the default mapping', async () => {
      const { el } = mount(reg, 'sui-isolation-badge', {
        tier: 'container',
        label: 'Sandboxed by Ship',
      })
      await flush()
      expect(shadow(el).querySelector('.label')?.textContent?.trim()).toBe(
        'Sandboxed by Ship'
      )
    })

    it('aria-label includes strategy when present', async () => {
      const { el } = mount(reg, 'sui-isolation-badge', {
        tier: 'container',
        strategy: 'docker',
      })
      await flush()
      expect(el.getAttribute('aria-label')).toContain('Container')
      expect(el.getAttribute('aria-label')).toContain('docker')
    })

    it('aria-label flags downgraded state', async () => {
      const { el } = mount(reg, 'sui-isolation-badge', {
        tier: 'none',
        downgraded: 'true',
      })
      await flush()
      expect(el.getAttribute('aria-label')).toContain('downgraded')
    })

    it('downgraded attribute reflects on the host', async () => {
      const { el } = mount(reg, 'sui-isolation-badge', {
        tier: 'none',
        downgraded: 'true',
      })
      await flush()
      expect(el.getAttribute('downgraded')).toBe('true')
    })

    it('title attribute mirrors aria-label for hover tooltips', async () => {
      const { el } = mount(reg, 'sui-isolation-badge', {
        tier: 'container',
        strategy: 'docker',
      })
      await flush()
      expect(el.title).toBe(el.getAttribute('aria-label'))
    })

    it('role=status is set on mount', async () => {
      const { el } = mount(reg, 'sui-isolation-badge', { tier: 'none' })
      await flush()
      expect(el.getAttribute('role')).toBe('status')
    })

    it('renders the ellipsis label when tier=unknown after the fetch resolves to nothing', async () => {
      const { el } = mount(reg, 'sui-isolation-badge', { tier: 'unknown' })
      // give the auto-fetch a tick to settle (mocked to 503 -> null)
      await new Promise((r) => setTimeout(r, 10))
      await flush()
      expect(shadow(el).querySelector('.label')?.textContent?.trim()).toBe('…')
    })
  })
  describe('auto-fetch from connect.isolation.status', () => {
    it('updates tier/strategy/downgraded from a successful fetch', async () => {
      // The Connect wire's `IsolationStatusResponse` is the typed
      // `IsolationDecision` envelope — `decision.resolved` carries the
      // (tier, strategy, downgraded) tuple the badge needs. Connect
      // POSTs the proto-encoded request to the wire path; the
      // interceptor reads `globalThis.fetch` (the same transport the
      // SUI runtime + the rest of the app use) and returns a
      // `Response` whose body is the proto JSON.
      vi.spyOn(globalThis, 'fetch').mockResolvedValue(
        new Response(
          JSON.stringify({
            capabilities: {},
            decision: {
              resolved: {
                strategy: 'docker',
                tier: 'container',
                downgraded: false,
              },
            },
          }),
          { status: 200, headers: { 'content-type': 'application/json' } }
        )
      )
      const { el } = mount(reg, 'sui-isolation-badge')
      await new Promise((r) => setTimeout(r, 30))
      await flush()
      expect(el.getAttribute('tier')).toBe('container')
      expect(el.getAttribute('strategy')).toBe('docker')
      expect(el.getAttribute('downgraded')).toBe('false')
    })

    it('falls back to unknown when fetch fails', async () => {
      vi.spyOn(globalThis, 'fetch').mockRejectedValue(new Error('network'))
      const { el } = mount(reg, 'sui-isolation-badge')
      await new Promise((r) => setTimeout(r, 30))
      await flush()
      // Element starts with no `tier` attribute (default is in the
      // serialized prop view, not the DOM attribute). The 'unknown'
      // value is what `tier()` returns; the host attribute may still
      // be absent. Verify the label renders the unknown placeholder.
      expect(shadow(el).querySelector('.label')?.textContent?.trim()).toBe('…')
    })

    it('does NOT auto-fetch when tier is supplied explicitly', async () => {
      vi.restoreAllMocks()
      const spy = vi.spyOn(globalThis, 'fetch')
      const { el } = mount(reg, 'sui-isolation-badge', { tier: 'container' })
      await new Promise((r) => setTimeout(r, 30))
      await flush()
      expect(spy).not.toHaveBeenCalled()
      expect(el.getAttribute('tier')).toBe('container')
    })
  })
})
