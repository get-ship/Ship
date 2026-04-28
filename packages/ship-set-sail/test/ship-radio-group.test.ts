import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-radio-group (sail)', () => {
  beforeAll(async () => {
    // Load child first so the group sees upgraded <ship-radio> at mount.
    await loadSailComponent('ship-radio.set')
    await loadSailComponent('ship-radio-group.set')
  }, 120_000)

  function makeGroup(attrs = ''): HTMLElement {
    const wrap = document.createElement('div')
    wrap.innerHTML = `
      <ship-radio-group ${attrs}>
        <ship-radio value="a">Option A</ship-radio>
        <ship-radio value="b">Option B</ship-radio>
        <ship-radio value="c" disabled>Option C</ship-radio>
      </ship-radio-group>
    `
    document.body.appendChild(wrap)
    return wrap.querySelector('ship-radio-group') as HTMLElement
  }

  async function settle() {
    await new Promise((r) => queueMicrotask(() => r(undefined)))
  }

  it('renders radios and sets role=radiogroup', async () => {
    const el = makeGroup()
    await settle()
    expect(el.getAttribute('role')).toBe('radiogroup')
    expect(el.querySelectorAll('ship-radio').length).toBe(3)
    el.parentElement!.remove()
  })

  it('group declares static formAssociated = true', () => {
    const ctor = customElements.get('ship-radio-group') as any
    expect(ctor.formAssociated).toBe(true)
  })

  it('selects radio matching value attribute and reflects checked', async () => {
    const el = makeGroup('value="b"')
    await settle()
    const radios = Array.from(el.querySelectorAll('ship-radio'))
    expect(radios[0]!.hasAttribute('checked')).toBe(false)
    expect(radios[1]!.hasAttribute('checked')).toBe(true)
    expect(radios[2]!.hasAttribute('checked')).toBe(false)
    // ARIA + roving tabindex on inner inputs.
    const inputs = radios.map(r => r.querySelector('input') as HTMLInputElement)
    expect(inputs[0]!.getAttribute('aria-checked')).toBe('false')
    expect(inputs[1]!.getAttribute('aria-checked')).toBe('true')
    expect(inputs[0]!.getAttribute('tabindex')).toBe('-1')
    expect(inputs[1]!.getAttribute('tabindex')).toBe('0')
    el.parentElement!.remove()
  })

  it('clicking a radio selects it, sets value, and emits ship-change', async () => {
    const el = makeGroup()
    await settle()
    const events: any[] = []
    el.addEventListener('ship-change', (e: any) => events.push(e.detail))
    const radioA = el.querySelector('ship-radio[value="a"]') as HTMLElement
    const inputA = radioA.querySelector('input') as HTMLInputElement
    inputA.click()
    expect(el.getAttribute('value')).toBe('a')
    expect(radioA.hasAttribute('checked')).toBe(true)
    expect(events).toEqual([{ value: 'a' }])
    el.parentElement!.remove()
  })

  it('arrow-down moves selection to next enabled radio', async () => {
    const el = makeGroup('value="a"')
    await settle()
    const radioA = el.querySelector('ship-radio[value="a"]') as HTMLElement
    const inputA = radioA.querySelector('input') as HTMLInputElement
    inputA.dispatchEvent(new KeyboardEvent('keydown', {
      key: 'ArrowDown', bubbles: true, composed: true, cancelable: true,
    }))
    expect(el.getAttribute('value')).toBe('b')
    const radioB = el.querySelector('ship-radio[value="b"]') as HTMLElement
    expect(radioB.hasAttribute('checked')).toBe(true)
    el.parentElement!.remove()
  })

  it('arrow-up wraps to last enabled radio', async () => {
    const el = makeGroup('value="a"')
    await settle()
    const radioA = el.querySelector('ship-radio[value="a"]') as HTMLElement
    const inputA = radioA.querySelector('input') as HTMLInputElement
    inputA.dispatchEvent(new KeyboardEvent('keydown', {
      key: 'ArrowUp', bubbles: true, composed: true, cancelable: true,
    }))
    // c is disabled, so wrap should land on b.
    expect(el.getAttribute('value')).toBe('b')
    el.parentElement!.remove()
  })

  it('disabled radio is skipped during navigation (wraps past it)', async () => {
    const el = makeGroup('value="b"')
    await settle()
    const radioB = el.querySelector('ship-radio[value="b"]') as HTMLElement
    const inputB = radioB.querySelector('input') as HTMLInputElement
    inputB.dispatchEvent(new KeyboardEvent('keydown', {
      key: 'ArrowDown', bubbles: true, composed: true, cancelable: true,
    }))
    expect(el.getAttribute('value')).toBe('a')
    el.parentElement!.remove()
  })

  it('group disabled propagates disabled to all radios and prevents selection', async () => {
    const el = makeGroup('disabled value="a"')
    await settle()
    const radios = Array.from(el.querySelectorAll('ship-radio'))
    for (const r of radios) expect(r.hasAttribute('disabled')).toBe(true)
    // Inputs are also disabled — clicking shouldn't change value.
    const radioB = el.querySelector('ship-radio[value="b"]') as HTMLElement
    const inputB = radioB.querySelector('input') as HTMLInputElement
    expect(inputB.disabled).toBe(true)
    el.parentElement!.remove()
  })

  it('inherits tone and size onto unrestricted children', async () => {
    const el = makeGroup('tone="success" size="lg"')
    await settle()
    const radios = Array.from(el.querySelectorAll('ship-radio'))
    for (const r of radios) {
      expect(r.getAttribute('tone')).toBe('success')
      expect(r.getAttribute('size')).toBe('lg')
    }
    el.parentElement!.remove()
  })

  it('explicit tone/size on a radio overrides group inheritance', async () => {
    const wrap = document.createElement('div')
    wrap.innerHTML = `
      <ship-radio-group tone="success" size="lg">
        <ship-radio value="a" tone="info">A</ship-radio>
        <ship-radio value="b" size="sm">B</ship-radio>
      </ship-radio-group>
    `
    document.body.appendChild(wrap)
    await settle()
    const a = wrap.querySelector('ship-radio[value="a"]')!
    const b = wrap.querySelector('ship-radio[value="b"]')!
    expect(a.getAttribute('tone')).toBe('info')
    expect(a.getAttribute('size')).toBe('lg')
    expect(b.getAttribute('tone')).toBe('success')
    expect(b.getAttribute('size')).toBe('sm')
    wrap.remove()
  })

  it('label prop becomes aria-label on the host', async () => {
    const el = makeGroup('label="Pick one"')
    await settle()
    expect(el.getAttribute('aria-label')).toBe('Pick one')
    el.parentElement!.remove()
  })

  it('form_reset restores defaultValue captured at mount', async () => {
    const form = document.createElement('form')
    form.innerHTML = `
      <ship-radio-group name="choice" value="b">
        <ship-radio value="a">A</ship-radio>
        <ship-radio value="b">B</ship-radio>
      </ship-radio-group>
    `
    document.body.appendChild(form)
    await settle()
    const group = form.querySelector('ship-radio-group') as HTMLElement
    // Change selection.
    const radioA = group.querySelector('ship-radio[value="a"]') as HTMLElement
    ;(radioA.querySelector('input') as HTMLInputElement).click()
    expect(group.getAttribute('value')).toBe('a')
    // jsdom 25 does not always invoke formResetCallback from form.reset(),
    // so the runtime exposes the lifecycle as the 'sail:form-reset' event.
    // Dispatch it directly to exercise the on-form_reset handler.
    group.dispatchEvent(new CustomEvent('sail:form-reset'))
    await settle()
    expect(group.getAttribute('value')).toBe('b')
    const radioB = group.querySelector('ship-radio[value="b"]') as HTMLElement
    expect(radioB.hasAttribute('checked')).toBe(true)
    expect(radioA.hasAttribute('checked')).toBe(false)
    form.remove()
  })

  it('form_restore loads value from saved state', async () => {
    const el = makeGroup('value="a"')
    await settle()
    el.dispatchEvent(new CustomEvent('sail:form-restore', {
      detail: { state: 'b', mode: 'restore' },
    }))
    await settle()
    expect(el.getAttribute('value')).toBe('b')
    const radioB = el.querySelector('ship-radio[value="b"]') as HTMLElement
    expect(radioB.hasAttribute('checked')).toBe(true)
    el.parentElement!.remove()
  })

  it('host can be attached inside a <form> without error', () => {
    const form = document.createElement('form')
    const group = document.createElement('ship-radio-group')
    group.setAttribute('name', 'pick')
    group.setAttribute('value', 'a')
    form.appendChild(group)
    document.body.appendChild(form)
    expect(() => form.elements).not.toThrow()
    form.remove()
  })

  it('clicking the same radio twice emits ship-change only once', async () => {
    const el = makeGroup()
    await settle()
    const events: any[] = []
    el.addEventListener('ship-change', (e: any) => events.push(e.detail))
    const inputA = (el.querySelector('ship-radio[value="a"]') as HTMLElement)
      .querySelector('input') as HTMLInputElement
    inputA.click()
    inputA.click()
    expect(events).toEqual([{ value: 'a' }])
    el.parentElement!.remove()
  })

  it('tone × size matrix renders without error', async () => {
    const tones = ['default', 'info', 'success', 'warning', 'destructive', 'accent', 'muted']
    const sizes = ['sm', 'md', 'lg']
    for (const tone of tones) {
      for (const size of sizes) {
        const wrap = document.createElement('div')
        wrap.innerHTML = `
          <ship-radio-group tone="${tone}" size="${size}" value="a">
            <ship-radio value="a">A</ship-radio>
            <ship-radio value="b">B</ship-radio>
          </ship-radio-group>
        `
        document.body.appendChild(wrap)
        await settle()
        const group = wrap.querySelector('ship-radio-group')!
        expect(group.querySelectorAll('ship-radio').length).toBe(2)
        wrap.remove()
      }
    }
  })
})
