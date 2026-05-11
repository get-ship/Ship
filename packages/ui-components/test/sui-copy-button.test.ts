import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import '../src/sui-copy-button/dist/sui-copy-button.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-copy-button>', () => {
  beforeEach(() => {
    Object.assign(navigator, {
      clipboard: { writeText: vi.fn().mockResolvedValue(undefined) },
    })
  })

  it('registers the custom element', () => {
    expect(customElements.get('sui-copy-button')).toBeDefined()
  })

  it('renders a button element', () => {
    const { el } = mount(reg, 'sui-copy-button', { value: 'x' })
    expect(shadow(el).querySelector('button[part="button"]')).not.toBeNull()
  })

  it('label prop shows in shadow text (via default-label span)', async () => {
    const { el } = mount(reg, 'sui-copy-button', { value: 'x', label: 'Copy me' })
    await flush()
    expect(shadow(el).textContent).toContain('Copy me')
  })

  it('default label is "Copy"', async () => {
    const { el } = mount(reg, 'sui-copy-button', { value: 'x' })
    await flush()
    expect(shadow(el).textContent).toContain('Copy')
  })

  it('click triggers navigator.clipboard.writeText with value', async () => {
    const { el } = mount(reg, 'sui-copy-button', { value: 'hello' })
    const btn = shadow(el).querySelector('button')!
    btn.click()
    await flush()
    expect(navigator.clipboard.writeText).toHaveBeenCalledWith('hello')
  })

  it('click dispatches sui-copy event with value detail', async () => {
    const { el } = mount(reg, 'sui-copy-button', { value: 'hello' })
    const listener = vi.fn()
    el.addEventListener('sui-copy', listener)
    shadow(el).querySelector('button')!.click()
    await flush()
    expect(listener).toHaveBeenCalledOnce()
    expect((listener.mock.calls[0][0] as CustomEvent).detail).toEqual({ value: 'hello' })
  })

  it('copied attribute is set after click', async () => {
    const { el } = mount(reg, 'sui-copy-button', { value: 'x' })
    shadow(el).querySelector('button')!.click()
    await flush()
    expect(el.hasAttribute('copied')).toBe(true)
  })

  it('default label changes to "Copied" after click', async () => {
    const { el } = mount(reg, 'sui-copy-button', { value: 'x', label: 'Copy' })
    shadow(el).querySelector('button')!.click()
    await flush()
    expect(shadow(el).textContent).toContain('Copied')
  })

  it('copied attribute resets after 1.2s timeout', async () => {
    vi.useFakeTimers()
    const { el } = mount(reg, 'sui-copy-button', { value: 'x' })
    shadow(el).querySelector('button')!.click()
    await Promise.resolve()
    await Promise.resolve()
    expect(el.hasAttribute('copied')).toBe(true)
    vi.advanceTimersByTime(1300)
    expect(el.hasAttribute('copied')).toBe(false)
    vi.useRealTimers()
  })

  it('disconnect clears pending reset timer', async () => {
    vi.useFakeTimers()
    const { el, container } = mount(reg, 'sui-copy-button', { value: 'x' })
    shadow(el).querySelector('button')!.click()
    await Promise.resolve()
    await Promise.resolve()
    container.remove()
    expect(() => vi.advanceTimersByTime(2000)).not.toThrow()
    vi.useRealTimers()
  })
})
