import { describe, expect, it } from 'vitest'
import { morph } from '../src/morph'

const fromHtml = (html: string): Element => {
  const tpl = document.createElement('template')
  tpl.innerHTML = html
  return tpl.content.firstElementChild as Element
}

describe('morph', () => {
  it('updates text content in place', () => {
    const t = fromHtml('<div>old</div>')
    const ref = t.firstChild!
    morph(t, fromHtml('<div>new</div>'))
    expect(t.textContent).toBe('new')
    expect(t.firstChild).toBe(ref) // preserved identity
  })

  it('adds new attributes', () => {
    const t = fromHtml('<div></div>')
    morph(t, fromHtml('<div class="x" id="y"></div>'))
    expect(t.getAttribute('class')).toBe('x')
    expect(t.getAttribute('id')).toBe('y')
  })

  it('removes gone attributes', () => {
    const t = fromHtml('<div class="x" id="y"></div>')
    morph(t, fromHtml('<div></div>'))
    expect(t.hasAttribute('class')).toBe(false)
    expect(t.hasAttribute('id')).toBe(false)
  })

  it('updates changed attributes', () => {
    const t = fromHtml('<a href="/old">x</a>')
    morph(t, fromHtml('<a href="/new">x</a>'))
    expect(t.getAttribute('href')).toBe('/new')
  })

  it('replaces element when tag differs', () => {
    const parent = document.createElement('div')
    const t = fromHtml('<span>x</span>')
    parent.appendChild(t)
    morph(parent.firstElementChild as Element, fromHtml('<b>x</b>'))
    expect(parent.firstElementChild!.tagName).toBe('B')
  })

  it('appends new children', () => {
    const t = fromHtml('<ul><li>a</li></ul>')
    morph(t, fromHtml('<ul><li>a</li><li>b</li></ul>'))
    expect(t.children.length).toBe(2)
    expect(t.children[1]!.textContent).toBe('b')
  })

  it('removes surplus children', () => {
    const t = fromHtml('<ul><li>a</li><li>b</li><li>c</li></ul>')
    morph(t, fromHtml('<ul><li>a</li></ul>'))
    expect(t.children.length).toBe(1)
  })

  it('preserves element identity across structural updates', () => {
    const t = fromHtml('<ul><li>a</li><li>b</li></ul>')
    const liA = t.children[0]!
    morph(t, fromHtml('<ul><li>A</li><li>B</li></ul>'))
    expect(t.children[0]).toBe(liA)
    expect(t.children[0]!.textContent).toBe('A')
  })

  it('input value sync', () => {
    const t = fromHtml('<input type="text" />') as HTMLInputElement
    t.value = 'typed'
    morph(t, fromHtml('<input type="text" value="server" />'))
    expect(t.value).toBe('server')
  })

  it('checkbox checked sync', () => {
    const t = fromHtml('<input type="checkbox" />') as HTMLInputElement
    morph(t, fromHtml('<input type="checkbox" checked />'))
    expect(t.checked).toBe(true)
  })

  it('skip option opts subtree out', () => {
    const t = fromHtml('<div><span data-keep>orig</span></div>')
    const span = t.firstElementChild!
    morph(t, fromHtml('<div><span data-keep>changed</span></div>'), {
      skip: (el) => el.hasAttribute('data-keep'),
    })
    expect(span.textContent).toBe('orig')
  })
})
