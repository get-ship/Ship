import { afterEach, describe, expect, it } from 'vitest'
import '../src/ship-watch-badge/ship-watch-badge.compiled.js'
import { createContainerRegistry, mount } from '@ship/test-setup'

// No setShipClient — ship-watch-badge is purely presentational and does not
// touch the SDK. The host flips the reflected `state` attribute and the
// component renders.

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const badge = (el: HTMLElement) =>
  shadow(el).querySelector('.badge') as HTMLElement | null
const dot = (el: HTMLElement) =>
  shadow(el).querySelector('.dot') as HTMLElement | null
const visibleLabel = (el: HTMLElement) => {
  const root = shadow(el)
  return Array.from(root.querySelectorAll<HTMLElement>('.l'))
    .find((node) => getComputedStyle(node).display !== 'none')?.textContent
}

describe('<ship-watch-badge>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-watch-badge')).toBeDefined()
  })

  it('renders the live state by default', () => {
    const { el } = mount(reg, 'ship-watch-badge')
    expect(badge(el)).not.toBeNull()
    expect(dot(el)).not.toBeNull()
    expect(visibleLabel(el)).toBe('live')
  })

  it('renders the connecting state with the connecting label', () => {
    const { el } = mount(reg, 'ship-watch-badge', { state: 'connecting' })
    expect(visibleLabel(el)).toBe('connecting…')
  })

  it('renders the error state', () => {
    const { el } = mount(reg, 'ship-watch-badge', { state: 'error' })
    expect(visibleLabel(el)).toBe('reconnecting')
  })

  it('uses the label prop when provided', () => {
    const { el } = mount(reg, 'ship-watch-badge', { label: 'live · 3s ago' })
    expect(visibleLabel(el)).toBe('live · 3s ago')
  })

  it('reflects the state attribute back to the DOM property', () => {
    const { el } = mount(reg, 'ship-watch-badge', { state: 'paused' })
    const host = el as HTMLElement & { state: string }
    expect(host.getAttribute('state')).toBe('paused')
    expect(host.state).toBe('paused')
  })
})
