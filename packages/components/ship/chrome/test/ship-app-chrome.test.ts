// Behavioral spec for <ship-app-chrome> — the per-app root layout
// chrome wrapper (the chrome the LAYOUTS table's `tag` field names for
// a root `routes/_layout.sui`, prefix '/').
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, it, expect } from 'vitest'
import { renderComponent } from '@ship/test-setup'
import '@ship/sui-primitives'
// <ship-app-chrome> composes <ship-nav-top>, which composes
// <sui-project-switcher> + <sui-account-menu> + <sui-theme-picker>.
// The whole chrome pack must register before the wrapper mounts;
// otherwise the project switcher renders a bare shadow.
import '@ship/chrome'
import controller from '../src/ship-app-chrome/ship-app-chrome.controller'

const SRC = readFileSync(
  resolve(__dirname, '../src/ship-app-chrome/ship-app-chrome.sui'),
  'utf8',
)
const TAG = 'ship-app-chrome'

function mount() {
  return renderComponent(SRC, TAG, { controller, attrs: {} })
}
const root = (c: { el: HTMLElement }) => c.el.shadowRoot ?? c.el

describe('<ship-app-chrome>', () => {
  it('composes the nav-top + a named route slot', () => {
    const c = mount()
    // The bar wrapper is the chrome's first child — its presence is
    // the unit-of-one for the nav being declared ONCE, in the chrome,
    // not per-page.
    const bar = root(c).querySelector('ship-nav-top') as HTMLElement | null
    expect(bar).toBeTruthy()
    // The main outlet exposes the named slot the runtime projects
    // the route content into. The runtime mounts a <div
    // slot="route"> into the chrome chain; the chrome's <slot
    // name="route"> is what fills it.
    const main = root(c).querySelector('main[part="main"]') as HTMLElement | null
    expect(main).toBeTruthy()
    const slot = main?.querySelector('slot[name="route"]') as HTMLSlotElement | null
    expect(slot).toBeTruthy()
    c.unmount()
  })
})
