import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-segmented/dist/sui-segmented-option.compiled.js'
import '../src/sui-segmented/dist/sui-segmented.compiled.js'
import { createContainerRegistry, flush } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const mountSegmented = (attrs = '', optionMarkup = '') => {
  const container = reg.mk()
  container.innerHTML = `
    <sui-segmented ${attrs}>
      ${optionMarkup || `
        <sui-segmented-option value="rows">Rows</sui-segmented-option>
        <sui-segmented-option value="schema">Schema</sui-segmented-option>
        <sui-segmented-option value="disabled-opt" disabled>Disabled</sui-segmented-option>
      `}
    </sui-segmented>
  `
  return container.querySelector('sui-segmented') as HTMLElement
}

// ── sui-segmented-option ───────────────────────────────────────────────────

describe('<sui-segmented-option>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-segmented-option')).toBeDefined()
  })

  it('reflects value attribute', () => {
    const container = reg.mk()
    container.innerHTML = '<sui-segmented-option value="foo">Foo</sui-segmented-option>'
    const el = container.querySelector('sui-segmented-option') as HTMLElement
    expect(el.getAttribute('value')).toBe('foo')
  })

  it('has a shadow root', () => {
    const container = reg.mk()
    container.innerHTML = '<sui-segmented-option value="foo">Foo</sui-segmented-option>'
    const el = container.querySelector('sui-segmented-option') as HTMLElement
    // JSDOM does not evaluate shadow CSS so display:none can't be asserted here,
    // but the element must still have a shadow root from the compiled template.
    expect(el.shadowRoot).not.toBeNull()
  })
})

// ── sui-segmented ──────────────────────────────────────────────────────────

describe('<sui-segmented>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-segmented')).toBeDefined()
  })

  it('sets role=radiogroup on host', async () => {
    const el = mountSegmented()
    await flush()
    expect(el.getAttribute('role')).toBe('radiogroup')
  })

  it('renders one button per option in shadow DOM', async () => {
    const el = mountSegmented()
    await flush()
    const btns = el.shadowRoot!.querySelectorAll('button[role="radio"]')
    expect(btns.length).toBe(3)
  })

  it('selected button has aria-checked=true', async () => {
    const el = mountSegmented('value="rows"')
    await flush()
    const btns = el.shadowRoot!.querySelectorAll('button[role="radio"]')
    expect(btns[0].getAttribute('aria-checked')).toBe('true')
    expect(btns[1].getAttribute('aria-checked')).toBe('false')
  })

  it('clicking a button selects it and emits sui-change', async () => {
    const el = mountSegmented('value="rows"')
    await flush()
    let detail: { value: string } | null = null
    el.addEventListener('sui-change', (e) => { detail = (e as CustomEvent).detail })
    const btns = el.shadowRoot!.querySelectorAll<HTMLButtonElement>('button[role="radio"]')
    btns[1].click()
    await flush()
    expect(el.getAttribute('value')).toBe('schema')
    expect(detail).toEqual({ value: 'schema' })
  })

  it('clicking already-selected option does not emit sui-change', async () => {
    const el = mountSegmented('value="rows"')
    await flush()
    let count = 0
    el.addEventListener('sui-change', () => count++)
    const btns = el.shadowRoot!.querySelectorAll<HTMLButtonElement>('button[role="radio"]')
    btns[0].click()
    await flush()
    expect(count).toBe(0)
  })

  it('disabled option button has aria-disabled=true and is not clickable', async () => {
    const el = mountSegmented('value="rows"')
    await flush()
    let fired = false
    el.addEventListener('sui-change', () => { fired = true })
    const btns = el.shadowRoot!.querySelectorAll<HTMLButtonElement>('button[role="radio"]')
    const disabledBtn = btns[2]
    expect(disabledBtn.getAttribute('aria-disabled')).toBe('true')
    expect(disabledBtn.disabled).toBe(true)
    disabledBtn.click()
    await flush()
    expect(fired).toBe(false)
  })

  it('host disabled blocks all options', async () => {
    const el = mountSegmented('value="rows" disabled')
    await flush()
    let fired = false
    el.addEventListener('sui-change', () => { fired = true })
    const btns = el.shadowRoot!.querySelectorAll<HTMLButtonElement>('button[role="radio"]')
    btns[1].click()
    await flush()
    expect(fired).toBe(false)
    expect(el.getAttribute('value')).toBe('rows')
  })

  it('ArrowRight moves to next option', async () => {
    const el = mountSegmented('value="rows"')
    await flush()
    const btns = el.shadowRoot!.querySelectorAll<HTMLButtonElement>('button[role="radio"]')
    btns[0].dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true }))
    await flush()
    expect(el.getAttribute('value')).toBe('schema')
  })

  it('ArrowLeft wraps to previous enabled option', async () => {
    const el = mountSegmented('value="rows"')
    await flush()
    const btns = el.shadowRoot!.querySelectorAll<HTMLButtonElement>('button[role="radio"]')
    btns[0].dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowLeft', bubbles: true }))
    await flush()
    expect(el.getAttribute('value')).toBe('schema')
  })

  it('size=sm reflects on host', () => {
    const el = mountSegmented('size="sm"')
    expect(el.getAttribute('size')).toBe('sm')
  })

  it('form participation: FormData contains name=value', async () => {
    const container = reg.mk()
    container.innerHTML = `
      <form>
        <sui-segmented name="view" value="rows">
          <sui-segmented-option value="rows">Rows</sui-segmented-option>
          <sui-segmented-option value="schema">Schema</sui-segmented-option>
        </sui-segmented>
      </form>
    `
    const form = container.querySelector('form') as HTMLFormElement
    await flush()
    const data = new FormData(form)
    expect(data.get('view')).toBe('rows')
  })

  it('form participation: FormData updates after selection', async () => {
    const container = reg.mk()
    container.innerHTML = `
      <form>
        <sui-segmented name="view" value="rows">
          <sui-segmented-option value="rows">Rows</sui-segmented-option>
          <sui-segmented-option value="schema">Schema</sui-segmented-option>
        </sui-segmented>
      </form>
    `
    const form = container.querySelector('form') as HTMLFormElement
    const seg = container.querySelector('sui-segmented') as HTMLElement
    await flush()
    const btns = seg.shadowRoot!.querySelectorAll<HTMLButtonElement>('button[role="radio"]')
    btns[1].click()
    await flush()
    const data = new FormData(form)
    expect(data.get('view')).toBe('schema')
  })
})
