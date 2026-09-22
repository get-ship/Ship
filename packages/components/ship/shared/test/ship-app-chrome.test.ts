import { afterEach, describe, expect, it } from 'vitest'
import '../src/ship-app-chrome/ship-app-chrome.compiled.js'
import { createContainerRegistry, mount } from '@ship/test-setup'

// <ship-app-chrome> is RUNTIME-FREE and purely presentational: it has no
// controller, dispatches no events, owns no navigation. It is the per-app root
// layout wrapper the LAYOUTS table names for a root `routes/_layout.sui`
// (prefix '/'). It composes @ship/chrome's <ship-nav-top> (which stays a
// runtime pack) and exposes the router's `route` slot. The composed bar renders
// as an unregistered element here — these tests assert the wrapper's own shadow
// structure, which is the dissolution contract.

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot ?? el

describe('<ship-app-chrome>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-app-chrome')).toBeDefined()
  })

  it('composes the nav-top — declared ONCE in the chrome, not per-page', () => {
    const { el } = mount(reg, 'ship-app-chrome')
    expect(shadow(el).querySelector('ship-nav-top')).not.toBeNull()
  })

  it('exposes the named route slot the runtime projects into', () => {
    const { el } = mount(reg, 'ship-app-chrome')
    const main = shadow(el).querySelector('main[part="main"]') as HTMLElement | null
    expect(main).not.toBeNull()
    expect(main!.querySelector('slot[name="route"]')).not.toBeNull()
  })

  it('forwards a command-bar slot through to the nav-top', () => {
    const { el } = mount(reg, 'ship-app-chrome')
    const fwd = shadow(el).querySelector(
      'ship-nav-top slot[name="command-bar"]',
    ) as HTMLSlotElement | null
    expect(fwd).not.toBeNull()
    expect(fwd!.getAttribute('slot')).toBe('command-bar')
  })
})
