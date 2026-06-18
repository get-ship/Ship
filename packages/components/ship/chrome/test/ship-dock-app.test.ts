// Behavioral spec for <ship-dock-app> — the dock tile launcher.
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, it, expect } from 'vitest'
import { renderComponent } from '@ship/test-setup'
import '@ship/sui-primitives'
import controller from '../src/ship-dock-app/ship-dock-app.controller'

const SRC = readFileSync(
  resolve(__dirname, '../src/ship-dock-app/ship-dock-app.sui'),
  'utf8',
)
const TAG = 'ship-dock-app'

function mount(attrs: Record<string, string | boolean>) {
  return renderComponent(SRC, TAG, { controller, attrs })
}
const root = (c: { el: HTMLElement }) => c.el.shadowRoot ?? c.el

describe('<ship-dock-app>', () => {
  it('reflects active and labels the tile', () => {
    const c = mount({ label: 'Paperback', active: true })
    expect(c.el.hasAttribute('active')).toBe(true)
    const app = root(c).querySelector('[part="app"]') as HTMLElement
    expect(app.getAttribute('title')).toBe('Paperback')
    c.unmount()
  })

  it('emits ship-launch{label} when the tile is clicked', () => {
    const c = mount({ label: 'Studio' })
    let detail: unknown = null
    c.el.addEventListener('ship-launch', (e) => { detail = (e as CustomEvent).detail })
    const app = root(c).querySelector('[part="app"]') as HTMLElement
    app.dispatchEvent(new Event('click', { bubbles: true }))
    expect(detail).toEqual({ label: 'Studio' })
    c.unmount()
  })
})
