import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-kbd/sui-kbd.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

describe('<sui-kbd>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-kbd')).toBeDefined()
  })

  it('sets role="group" on the host', () => {
    const { el } = mount(reg, 'sui-kbd')
    expect(el.getAttribute('role')).toBe('group')
  })

  it('sets aria-label when keys is provided', async () => {
    const { el } = mount(reg, 'sui-kbd', { keys: 'Cmd K' })
    await flush()
    expect(el.getAttribute('aria-label')).toBe('Keyboard shortcut: Cmd K')
  })

  it('omits aria-label when keys is empty', () => {
    const { el } = mount(reg, 'sui-kbd')
    expect(el.hasAttribute('aria-label')).toBe(false)
  })

  it('updates aria-label when keys attribute changes', async () => {
    const { el } = mount(reg, 'sui-kbd', { keys: 'Ctrl S' })
    await flush()
    expect(el.getAttribute('aria-label')).toBe('Keyboard shortcut: Ctrl S')
    el.setAttribute('keys', 'Ctrl Z')
    await flush()
    expect(el.getAttribute('aria-label')).toBe('Keyboard shortcut: Ctrl Z')
  })

  it('removes aria-label when keys is cleared', async () => {
    const { el } = mount(reg, 'sui-kbd', { keys: 'Cmd K' })
    await flush()
    expect(el.hasAttribute('aria-label')).toBe(true)
    el.setAttribute('keys', '')
    await flush()
    expect(el.hasAttribute('aria-label')).toBe(false)
  })
})
