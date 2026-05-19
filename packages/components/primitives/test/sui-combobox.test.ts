import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-combobox/sui-combobox.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const inputOf = (el: HTMLElement) => shadow(el).querySelector('input[role="combobox"]') as HTMLInputElement
const listOf = (el: HTMLElement) => shadow(el).querySelector('ul[role="listbox"]') as HTMLUListElement | null
const opts = (el: HTMLElement) => Array.from(shadow(el).querySelectorAll('li.opt[data-value]')) as HTMLLIElement[]

describe('<sui-combobox>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-combobox')).toBeDefined()
  })

  it('renders a shadow root with a combobox input', async () => {
    const { el } = mount(reg, 'sui-combobox')
    await flush()
    expect(el.shadowRoot).not.toBeNull()
    const inp = inputOf(el)
    expect(inp).not.toBeNull()
    expect(inp.getAttribute('aria-expanded')).toBe('false')
    expect(inp.getAttribute('aria-autocomplete')).toBe('list')
  })

  it('parses light-DOM <option> children into options', async () => {
    const { el } = mount(reg, 'sui-combobox', {}, '<option value="a">Alpha</option><option value="b">Beta</option>')
    await flush()
    inputOf(el).focus()
    await flush()
    expect(opts(el).map((o) => o.dataset.value)).toEqual(['a', 'b'])
  })

  it('parses options="…JSON…" attribute', async () => {
    const { el } = mount(reg, 'sui-combobox', { options: JSON.stringify([{ value: 'x', label: 'Xray' }, { value: 'y', label: 'Yankee' }]) })
    await flush()
    inputOf(el).focus()
    await flush()
    expect(opts(el).map((o) => o.dataset.value)).toEqual(['x', 'y'])
  })

  it('filters options by case-insensitive label substring', async () => {
    const { el } = mount(reg, 'sui-combobox', {}, '<option value="a">Apple</option><option value="b">Apricot</option><option value="c">Banana</option>')
    await flush()
    const inp = inputOf(el)
    inp.focus()
    await flush()
    inp.value = 'AP'
    inp.dispatchEvent(new Event('input', { bubbles: true }))
    await flush()
    expect(opts(el).map((o) => o.dataset.value)).toEqual(['a', 'b'])
  })

  it('reflects `value` attribute round-trip', async () => {
    const { el } = mount(reg, 'sui-combobox', { value: 'foo' })
    await flush()
    expect(el.getAttribute('value')).toBe('foo')
    el.setAttribute('value', 'bar')
    await flush()
    expect(el.getAttribute('value')).toBe('bar')
  })
})
