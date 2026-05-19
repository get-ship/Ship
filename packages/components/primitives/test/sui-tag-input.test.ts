import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-tag-input/sui-tag-input.compiled.js'
import '../src/sui-chip/sui-chip.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const inputOf = (el: HTMLElement) => shadow(el).querySelector('input.entry') as HTMLInputElement
const chipsOf = (el: HTMLElement) => Array.from(shadow(el).querySelectorAll('sui-chip')) as HTMLElement[]

describe('<sui-tag-input>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-tag-input')).toBeDefined()
  })

  it('renders a shadow root with an input and chip slot wrap', async () => {
    const { el } = mount(reg, 'sui-tag-input')
    await flush()
    expect(el.shadowRoot).not.toBeNull()
    expect(shadow(el).querySelector('.wrap')).not.toBeNull()
    expect(inputOf(el)).not.toBeNull()
  })

  it('parses comma-separated `value` into chips', async () => {
    const { el } = mount(reg, 'sui-tag-input', { value: 'red, green, blue' })
    await flush()
    expect(chipsOf(el)).toHaveLength(3)
    expect(chipsOf(el).map((c) => c.textContent?.trim())).toEqual(['red', 'green', 'blue'])
  })

  it('reflects attribute updates: value attr → chip list', async () => {
    const { el } = mount(reg, 'sui-tag-input', { value: 'a' })
    await flush()
    expect(chipsOf(el)).toHaveLength(1)
    el.setAttribute('value', 'a,b,c')
    await flush()
    expect(chipsOf(el)).toHaveLength(3)
    expect(el.getAttribute('value')).toBe('a,b,c')
  })

  it('Enter on input adds a tag and clears entry', async () => {
    const { el } = mount(reg, 'sui-tag-input')
    await flush()
    const input = inputOf(el)
    input.value = 'hello'
    input.dispatchEvent(new Event('input', { bubbles: true }))
    await flush()
    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true }))
    await flush()
    expect(chipsOf(el).map((c) => c.textContent?.trim())).toContain('hello')
    expect(input.value).toBe('')
  })

  it('Backspace on empty input removes the last tag', async () => {
    const { el } = mount(reg, 'sui-tag-input', { value: 'one,two' })
    await flush()
    const input = inputOf(el)
    input.value = ''
    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Backspace', bubbles: true, cancelable: true }))
    await flush()
    expect(chipsOf(el).map((c) => c.textContent?.trim())).toEqual(['one'])
  })

  it('dispatches sui-change with current tags array', async () => {
    const { el } = mount(reg, 'sui-tag-input')
    await flush()
    const events: string[][] = []
    el.addEventListener('sui-change', (e) => {
      events.push((e as CustomEvent<{ tags: string[] }>).detail.tags)
    })
    const input = inputOf(el)
    input.value = 'x'
    input.dispatchEvent(new Event('input', { bubbles: true }))
    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true }))
    await flush()
    expect(events.at(-1)).toEqual(['x'])
  })
})
