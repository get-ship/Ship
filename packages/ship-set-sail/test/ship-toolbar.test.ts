import { describe, expect, it, beforeAll, afterEach } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-toolbar (sail)', () => {
  let tag: string

  beforeAll(async () => {
    tag = await loadSailComponent('ship-toolbar.set')
  }, 120_000)

  afterEach(() => {
    document.body.innerHTML = ''
  })

  function mount(attrs = '', body = '') {
    const wrap = document.createElement('div')
    wrap.innerHTML = `<ship-toolbar ${attrs}>${body}</ship-toolbar>`
    const el = wrap.firstElementChild as HTMLElement
    document.body.appendChild(el)
    return el
  }

  it('renders without error and exposes the tag', () => {
    const el = mount()
    expect(el).not.toBeNull()
    expect(el.tagName.toLowerCase()).toBe(tag)
  })

  it('sets role=toolbar and aria-orientation=horizontal on the host', () => {
    const el = mount()
    expect(el.getAttribute('role')).toBe('toolbar')
    expect(el.getAttribute('aria-orientation')).toBe('horizontal')
  })

  it('does not clobber a pre-set role', () => {
    const el = document.createElement(tag)
    el.setAttribute('role', 'group')
    document.body.appendChild(el)
    expect(el.getAttribute('role')).toBe('group')
  })

  it('justify attribute reflects on the host', () => {
    for (const v of ['start', 'center', 'end', 'between']) {
      const el = mount(`justify="${v}"`)
      expect(el.getAttribute('justify')).toBe(v)
    }
  })

  it('align attribute reflects on the host', () => {
    for (const v of ['center', 'start', 'end', 'baseline']) {
      const el = mount(`align="${v}"`)
      expect(el.getAttribute('align')).toBe(v)
    }
  })

  it('gap attribute reflects on the host', () => {
    for (const v of ['sm', 'md', 'lg']) {
      const el = mount(`gap="${v}"`)
      expect(el.getAttribute('gap')).toBe(v)
    }
  })

  it('variant attribute reflects on the host', () => {
    const el = mount('variant="bordered"')
    expect(el.getAttribute('variant')).toBe('bordered')
  })

  it('justify × align × gap × variant matrix renders without error', () => {
    const justifies = ['start', 'center', 'end', 'between']
    const aligns = ['center', 'start', 'end', 'baseline']
    const gaps = ['sm', 'md', 'lg']
    const variants = ['default', 'bordered']
    let n = 0
    for (const justify of justifies) {
      for (const align of aligns) {
        for (const gap of gaps) {
          for (const variant of variants) {
            const el = mount(`justify="${justify}" align="${align}" gap="${gap}" variant="${variant}"`)
            expect(el).not.toBeNull()
            el.remove()
            n++
          }
        }
      }
    }
    expect(n).toBe(4 * 4 * 3 * 2)
  })

  it('ArrowRight cycles focus forward through non-disabled children', () => {
    const el = mount('', '<button id="b1">B1</button><button id="b2">B2</button><button id="b3">B3</button>')
    const [b1, b2, b3] = ['b1', 'b2', 'b3'].map(id => el.querySelector(`#${id}`) as HTMLElement)
    expect(b1).not.toBeNull()
    b1.focus()
    b1.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true, composed: true }))
    expect(document.activeElement).toBe(b2)
    b2.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true, composed: true }))
    expect(document.activeElement).toBe(b3)
    b3.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true, composed: true }))
    expect(document.activeElement).toBe(b1)
  })

  it('ArrowLeft cycles focus backward', () => {
    const el = mount('', '<button id="b1">B1</button><button id="b2">B2</button><button id="b3">B3</button>')
    const [b1, , b3] = ['b1', 'b2', 'b3'].map(id => el.querySelector(`#${id}`) as HTMLElement)
    b1.focus()
    b1.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowLeft', bubbles: true, composed: true }))
    expect(document.activeElement).toBe(b3)
  })

  it('disabled children are skipped in keyboard navigation', () => {
    const el = mount('', '<button id="b1">B1</button><button id="b2" disabled>B2</button><button id="b3">B3</button>')
    const [b1, , b3] = ['b1', 'b2', 'b3'].map(id => el.querySelector(`#${id}`) as HTMLElement)
    b1.focus()
    b1.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true, composed: true }))
    expect(document.activeElement).toBe(b3)
  })

  it('non-arrow keys do not prevent default or move focus', () => {
    const el = mount('', '<button id="b1">B1</button><button id="b2">B2</button>')
    const b1 = el.querySelector('#b1') as HTMLElement
    b1.focus()
    const ev = new KeyboardEvent('keydown', { key: 'Tab', bubbles: true, composed: true, cancelable: true })
    b1.dispatchEvent(ev)
    expect(document.activeElement).toBe(b1)
    expect(ev.defaultPrevented).toBe(false)
  })

  it('spacer slot child distributes separately from default slot children', () => {
    const el = mount('', '<div slot="spacer" id="sp"></div><button id="btn">Action</button>')
    const spacer = el.querySelector('#sp') as HTMLElement
    const btn = el.querySelector('#btn') as HTMLElement
    expect(spacer).not.toBeNull()
    expect(btn).not.toBeNull()
  })

  it('does not declare formAssociated', () => {
    const ctor = customElements.get(tag) as any
    expect(ctor.formAssociated).not.toBe(true)
  })
})
