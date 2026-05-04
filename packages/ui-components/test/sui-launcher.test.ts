import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-launcher/sui-launcher.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-launcher>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-launcher')).toBeDefined()
  })

  it('panel is hidden when open=false (default)', () => {
    const { el } = mount(reg, 'sui-launcher')
    expect(el.hasAttribute('open')).toBe(false)
    const panel = shadow(el).querySelector('.launcher-panel') as HTMLElement | null
    expect(panel).not.toBeNull()
    // CSS hides it — check attribute state
    expect(el.getAttribute('open')).toBeNull()
  })

  it('reflects [open] attribute when open prop is set', async () => {
    const { el } = mount(reg, 'sui-launcher', { open: true })
    await flush()
    expect(el.hasAttribute('open')).toBe(true)
  })

  it('panel and mask exist in shadow DOM', () => {
    const { el } = mount(reg, 'sui-launcher')
    expect(shadow(el).querySelector('[part~="panel"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="mask"]')).not.toBeNull()
  })

  it('has filter input in shadow DOM', () => {
    const { el } = mount(reg, 'sui-launcher')
    const input = shadow(el).querySelector<HTMLInputElement>('.launcher-filter-input')
    expect(input).not.toBeNull()
    expect(input!.getAttribute('aria-label')).toBe('Filter views')
  })

  it('filter input has correct placeholder', () => {
    const { el } = mount(reg, 'sui-launcher')
    const input = shadow(el).querySelector<HTMLInputElement>('.launcher-filter-input')!
    expect(input.placeholder).toBe('Filter views…')
  })

  it('body, head, and foot parts exist', () => {
    const { el } = mount(reg, 'sui-launcher')
    expect(shadow(el).querySelector('[part~="head"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="body"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="foot"]')).not.toBeNull()
  })

  it('panel has role=dialog and aria-modal=true', () => {
    const { el } = mount(reg, 'sui-launcher')
    const panel = shadow(el).querySelector('[part~="panel"]')!
    expect(panel.getAttribute('role')).toBe('dialog')
    expect(panel.getAttribute('aria-modal')).toBe('true')
  })

  it('sui:launcher:open event on document opens the launcher', async () => {
    const { el } = mount(reg, 'sui-launcher')
    await flush()
    expect(el.hasAttribute('open')).toBe(false)
    document.dispatchEvent(new CustomEvent('sui:launcher:open'))
    await flush()
    expect(el.hasAttribute('open')).toBe(true)
  })

  it('sui:launcher:close event on document closes the launcher', async () => {
    const { el } = mount(reg, 'sui-launcher', { open: true })
    await flush()
    document.dispatchEvent(new CustomEvent('sui:launcher:close'))
    await flush()
    expect(el.hasAttribute('open')).toBe(false)
  })

  it('Escape key closes launcher when open', async () => {
    const { el } = mount(reg, 'sui-launcher', { open: true })
    await flush()
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    await flush()
    expect(el.hasAttribute('open')).toBe(false)
  })

  it('Escape key does nothing when launcher is closed', async () => {
    const { el } = mount(reg, 'sui-launcher')
    await flush()
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    await flush()
    expect(el.hasAttribute('open')).toBe(false)
  })

  it('foot renders keyboard hint badges', async () => {
    const { el } = mount(reg, 'sui-launcher')
    await flush()
    const foot = shadow(el).querySelector('[part~="foot"]')!
    const kbds = foot.querySelectorAll('sui-kbd')
    expect(kbds.length).toBeGreaterThanOrEqual(1)
  })

  it('foot renders Install view link', async () => {
    const { el } = mount(reg, 'sui-launcher')
    await flush()
    const foot = shadow(el).querySelector('[part~="foot"]')!
    expect(foot.querySelector('.launcher-add')).not.toBeNull()
  })

  it('setting open=false after open=true clears the filter input', async () => {
    const { el } = mount(reg, 'sui-launcher', { open: true })
    await flush()
    const input = shadow(el).querySelector<HTMLInputElement>('.launcher-filter-input')!
    input.value = 'pipeline'
    // Close
    el.removeAttribute('open')
    await flush()
    expect(input.value).toBe('')
  })

  it('mask click closes the launcher', async () => {
    const { el } = mount(reg, 'sui-launcher', { open: true })
    await flush()
    const mask = shadow(el).querySelector<HTMLElement>('[part~="mask"]')!
    mask.click()
    await flush()
    expect(el.hasAttribute('open')).toBe(false)
  })

  it('count span starts at 0 before views load', async () => {
    const { el } = mount(reg, 'sui-launcher', { open: true })
    await flush()
    const count = shadow(el).querySelector('.launcher-count')!
    expect(count.textContent).toBe('0')
  })
})
