import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-select (sail)', () => {
  let tag: string

  beforeAll(async () => {
    tag = await loadSailComponent('ship-select.set')
  }, 120_000)

  function mount(markup: string): HTMLElement {
    const wrap = document.createElement('div')
    wrap.innerHTML = markup
    const el = wrap.firstElementChild as HTMLElement
    document.body.appendChild(el)
    return el
  }
  const flushMicro = () => new Promise<void>((r) => queueMicrotask(() => r()))

  it('renders a native select wrapped in a .field container', () => {
    const el = mount(`<${tag}><option value="a">A</option></${tag}>`)
    const sel = el.querySelector('select') as HTMLSelectElement
    const field = el.querySelector('.field')
    expect(sel).not.toBeNull()
    expect(field).not.toBeNull()
    el.remove()
  })

  it('mirrors slotted <option> children into the native select', async () => {
    const el = mount(
      `<${tag} value="b"><option value="a">A</option><option value="b">B</option></${tag}>`,
    )
    await flushMicro()
    const sel = el.querySelector('select') as HTMLSelectElement
    expect(sel.options.length).toBe(2)
    expect(sel.value).toBe('b')
    el.remove()
  })

  it('preserves <optgroup> structure', async () => {
    const el = mount(
      `<${tag} value="ny"><optgroup label="US"><option value="ny">New York</option><option value="la">LA</option></optgroup></${tag}>`,
    )
    await flushMicro()
    const sel = el.querySelector('select') as HTMLSelectElement
    expect(sel.querySelector('optgroup')).not.toBeNull()
    expect(sel.options.length).toBe(2)
    expect(sel.value).toBe('ny')
    el.remove()
  })

  it('renders the chrome label matching the current option text', async () => {
    const el = mount(
      `<${tag} value="b"><option value="a">Apple</option><option value="b">Banana</option></${tag}>`,
    )
    await flushMicro()
    const valueEl = el.querySelector('.value')!
    expect(valueEl.textContent).toBe('Banana')
    el.remove()
  })

  it('falls back to the first option when value does not match', async () => {
    const el = mount(
      `<${tag}><option value="a">Apple</option><option value="b">Banana</option></${tag}>`,
    )
    await flushMicro()
    const valueEl = el.querySelector('.value')!
    expect(valueEl.textContent).toBe('Apple')
    el.remove()
  })

  it('emits ship-change with the new value on selection', async () => {
    const el = mount(
      `<${tag}><option value="x">X</option><option value="y">Y</option></${tag}>`,
    )
    await flushMicro()
    const events: { value: string }[] = []
    el.addEventListener('ship-change', (e) => events.push((e as CustomEvent).detail))

    const sel = el.querySelector('select') as HTMLSelectElement
    sel.value = 'y'
    sel.dispatchEvent(new Event('change', { bubbles: true }))
    expect(events).toEqual([{ value: 'y' }])
    expect(el.querySelector('.value')!.textContent).toBe('Y')
    el.remove()
  })

  it('toggles data-placeholder when the selected option has empty value', async () => {
    const el = mount(
      `<${tag}><option value="">Pick one</option><option value="a">A</option></${tag}>`,
    )
    await flushMicro()
    expect(el.hasAttribute('data-placeholder')).toBe(true)

    const sel = el.querySelector('select') as HTMLSelectElement
    sel.value = 'a'
    sel.dispatchEvent(new Event('change', { bubbles: true }))
    expect(el.hasAttribute('data-placeholder')).toBe(false)
    el.remove()
  })

  it('name / required / disabled propagate to the native select', async () => {
    const el = mount(
      `<${tag} name="fruit" required disabled><option value="a">A</option></${tag}>`,
    )
    await flushMicro()
    const sel = el.querySelector('select') as HTMLSelectElement
    expect(sel.name).toBe('fruit')
    expect(sel.required).toBe(true)
    expect(sel.disabled).toBe(true)
    el.remove()
  })

  it('invalid sets aria-invalid on the native select', () => {
    const el = mount(
      `<${tag} invalid><option value="a">A</option></${tag}>`,
    )
    const sel = el.querySelector('select') as HTMLSelectElement
    expect(sel.getAttribute('aria-invalid')).toBe('true')
    el.remove()
  })

  it('label / described-by flow to aria-label / aria-describedby', () => {
    const el = mount(
      `<${tag} label="Choose a fruit" described-by="hint-1"><option value="a">A</option></${tag}>`,
    )
    const sel = el.querySelector('select') as HTMLSelectElement
    expect(sel.getAttribute('aria-label')).toBe('Choose a fruit')
    expect(sel.getAttribute('aria-describedby')).toBe('hint-1')
    el.remove()
  })

  it('component declares static formAssociated = true', () => {
    const ctor = customElements.get(tag) as { formAssociated?: boolean }
    expect(ctor.formAssociated).toBe(true)
  })

  it('host can be attached inside a <form> without error', () => {
    const el = mount(
      `<${tag} name="fruit"><option value="a">A</option></${tag}>`,
    )
    el.remove()
    const form = document.createElement('form')
    form.appendChild(el)
    document.body.appendChild(form)
    expect(() => form.elements).not.toThrow()
    form.remove()
  })

  it('tone × size matrix renders without error', async () => {
    const tones = ['default', 'info', 'success', 'warning', 'destructive', 'accent', 'muted']
    const sizes = ['sm', 'md', 'lg']
    let n = 0
    for (const tone of tones) {
      for (const size of sizes) {
        const el = mount(
          `<${tag} tone="${tone}" size="${size}"><option value="a">A</option></${tag}>`,
        )
        await flushMicro()
        expect(el.querySelector('select')).not.toBeNull()
        el.remove()
        n++
      }
    }
    expect(n).toBe(7 * 3)
  })
})
