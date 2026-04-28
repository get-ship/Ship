import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-slider (sail)', () => {
  let tag: string

  beforeAll(async () => {
    tag = await loadSailComponent('ship-slider.set')
  }, 120_000)

  function mount(attrs = ''): HTMLElement {
    const wrap = document.createElement('div')
    wrap.innerHTML = `<${tag} ${attrs}></${tag}>`
    const el = wrap.firstElementChild as HTMLElement
    document.body.appendChild(el)
    return el
  }
  const settle = () => new Promise<void>((r) => queueMicrotask(() => r()))

  it('renders track + fill + thumb chrome and a native range input', () => {
    const el = mount()
    expect(el.querySelector('.track')).not.toBeNull()
    expect(el.querySelector('.fill')).not.toBeNull()
    expect(el.querySelector('.thumb')).not.toBeNull()
    const input = el.querySelector('input') as HTMLInputElement
    expect(input).not.toBeNull()
    expect(input.type).toBe('range')
    el.remove()
  })

  it('default props yield 0/0/100/1 with empty fill', () => {
    const el = mount()
    const input = el.querySelector('input') as HTMLInputElement
    expect(input.min).toBe('0')
    expect(input.max).toBe('100')
    expect(input.step).toBe('1')
    expect(input.value).toBe('0')
    const fill = el.querySelector('.fill') as HTMLElement
    expect(fill.style.width).toBe('0%')
    el.remove()
  })

  it('value attribute flows to native input and fill width', () => {
    const el = mount('value="50" min="0" max="100"')
    const input = el.querySelector('input') as HTMLInputElement
    expect(input.value).toBe('50')
    const fill = el.querySelector('.fill') as HTMLElement
    expect(fill.style.width).toBe('50%')
    el.remove()
  })

  it('min / max / step propagate to the native input', () => {
    const el = mount('min="-5" max="25" step="0.5" value="10"')
    const input = el.querySelector('input') as HTMLInputElement
    expect(input.min).toBe('-5')
    expect(input.max).toBe('25')
    expect(input.step).toBe('0.5')
    expect(input.value).toBe('10')
    // (10 - -5) / (25 - -5) = 15 / 30 = 50%
    const fill = el.querySelector('.fill') as HTMLElement
    expect(fill.style.width).toBe('50%')
    el.remove()
  })

  it('clamps fill width to [0, 100] when value is out of range', () => {
    const el = mount('value="500" min="0" max="100"')
    const fill = el.querySelector('.fill') as HTMLElement
    expect(fill.style.width).toBe('100%')
    el.remove()

    const el2 = mount('value="-10" min="0" max="100"')
    const fill2 = el2.querySelector('.fill') as HTMLElement
    expect(fill2.style.width).toBe('0%')
    el2.remove()
  })

  it('native input event updates value, fill, and emits ship-change', () => {
    const el = mount('value="10" step="5"')
    const events: { value: number }[] = []
    el.addEventListener('ship-change', (e) => events.push((e as CustomEvent).detail))

    const input = el.querySelector('input') as HTMLInputElement
    input.value = '25'
    input.dispatchEvent(new Event('input', { bubbles: true }))
    expect(events).toEqual([{ value: 25 }])
    const fill = el.querySelector('.fill') as HTMLElement
    expect(fill.style.width).toBe('25%')
    el.remove()
  })

  it('does not emit ship-change when input fires with the same value', () => {
    const el = mount('value="10"')
    const events: unknown[] = []
    el.addEventListener('ship-change', (e) => events.push((e as CustomEvent).detail))
    const input = el.querySelector('input') as HTMLInputElement
    input.value = '10'
    input.dispatchEvent(new Event('input', { bubbles: true }))
    expect(events).toEqual([])
    el.remove()
  })

  it('disabled propagates to the native input', () => {
    const el = mount('disabled value="50"')
    const input = el.querySelector('input') as HTMLInputElement
    expect(input.disabled).toBe(true)
    expect(input.getAttribute('aria-disabled')).toBe('true')
    el.remove()
  })

  it('label flows to aria-label on the native input', () => {
    const el = mount('label="Volume"')
    const input = el.querySelector('input') as HTMLInputElement
    expect(input.getAttribute('aria-label')).toBe('Volume')
    el.remove()
  })

  it('aria-label falls back to "slider" when no label is set', () => {
    const el = mount()
    const input = el.querySelector('input') as HTMLInputElement
    expect(input.getAttribute('aria-label')).toBe('slider')
    el.remove()
  })

  it('name propagates to the native input for form submission', () => {
    const el = mount('name="volume" value="40"')
    const input = el.querySelector('input') as HTMLInputElement
    expect(input.name).toBe('volume')
    el.remove()
  })

  it('component declares static formAssociated = true', () => {
    const ctor = customElements.get(tag) as { formAssociated?: boolean }
    expect(ctor.formAssociated).toBe(true)
  })

  it('host can be attached inside a <form> without error', () => {
    const form = document.createElement('form')
    const slider = document.createElement(tag)
    slider.setAttribute('name', 'vol')
    slider.setAttribute('value', '30')
    form.appendChild(slider)
    document.body.appendChild(form)
    expect(() => form.elements).not.toThrow()
    form.remove()
  })

  it('form_reset restores defaultValue captured at mount', async () => {
    const form = document.createElement('form')
    form.innerHTML = `<${tag} name="vol" value="40"></${tag}>`
    document.body.appendChild(form)
    const slider = form.querySelector(tag) as HTMLElement
    await settle()

    const input = slider.querySelector('input') as HTMLInputElement
    input.value = '75'
    input.dispatchEvent(new Event('input', { bubbles: true }))
    expect(input.value).toBe('75')

    slider.dispatchEvent(new CustomEvent('sail:form-reset'))
    await settle()
    expect(input.value).toBe('40')
    const fill = slider.querySelector('.fill') as HTMLElement
    expect(fill.style.width).toBe('40%')
    form.remove()
  })

  it('form_restore loads the saved state value', async () => {
    const el = mount('value="20"')
    await settle()
    el.dispatchEvent(new CustomEvent('sail:form-restore', {
      detail: { state: '65', mode: 'restore' },
    }))
    await settle()
    const input = el.querySelector('input') as HTMLInputElement
    expect(input.value).toBe('65')
    const fill = el.querySelector('.fill') as HTMLElement
    expect(fill.style.width).toBe('65%')
    el.remove()
  })

  it('value updates after construction reflect to native input and fill', async () => {
    const el = mount('value="20"')
    el.setAttribute('value', '80')
    await settle()
    const input = el.querySelector('input') as HTMLInputElement
    expect(input.value).toBe('80')
    const fill = el.querySelector('.fill') as HTMLElement
    expect(fill.style.width).toBe('80%')
    el.remove()
  })

  it('thumb position tracks the fill percentage', () => {
    const el = mount('value="25"')
    const thumb = el.querySelector('.thumb') as HTMLElement
    expect(thumb.style.left).toBe('25%')
    el.remove()
  })

  it('tone × size matrix renders without error', () => {
    const tones = ['default', 'info', 'success', 'warning', 'destructive', 'accent', 'muted']
    const sizes = ['sm', 'md', 'lg']
    let n = 0
    for (const tone of tones) {
      for (const size of sizes) {
        const el = mount(`tone="${tone}" size="${size}" value="50"`)
        expect(el.querySelector('.thumb')).not.toBeNull()
        expect(el.querySelector('input')).not.toBeNull()
        el.remove()
        n++
      }
    }
    expect(n).toBe(7 * 3)
  })
})
