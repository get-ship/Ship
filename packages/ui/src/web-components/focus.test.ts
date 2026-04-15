import { afterEach, describe, expect, it } from 'vitest'
import { cleanupTestDom } from './util'
import { FocusReturn, focusableWithin, trapFocus } from './focus'

describe('focus utilities', () => {
  afterEach(cleanupTestDom)

  it('focusableWithin returns focusables in DOM order, skipping disabled', () => {
    document.body.insertAdjacentHTML(
      'beforeend',
      `<div id="root">
         <button>a</button>
         <button disabled>b</button>
         <a href="#">c</a>
         <input type="hidden" />
         <input />
       </div>`,
    )
    const root = document.getElementById('root')!
    const items = focusableWithin(root).map((n) => n.tagName.toLowerCase())
    expect(items).toEqual(['button', 'a', 'input'])
  })

  it('trapFocus wraps Shift+Tab from first → last and Tab from last → first', () => {
    document.body.insertAdjacentHTML(
      'beforeend',
      `<div id="root"><button id="a">a</button><button id="b">b</button></div>`,
    )
    const root = document.getElementById('root')!
    const a = document.getElementById('a') as HTMLButtonElement
    const b = document.getElementById('b') as HTMLButtonElement

    a.focus()
    const back = new KeyboardEvent('keydown', { key: 'Tab', shiftKey: true, cancelable: true })
    trapFocus(root, back)
    expect(back.defaultPrevented).toBe(true)
    expect(document.activeElement).toBe(b)

    b.focus()
    const fwd = new KeyboardEvent('keydown', { key: 'Tab', cancelable: true })
    trapFocus(root, fwd)
    expect(fwd.defaultPrevented).toBe(true)
    expect(document.activeElement).toBe(a)
  })

  it('FocusReturn saves and restores focus', () => {
    document.body.insertAdjacentHTML(
      'beforeend',
      `<button id="outer">outer</button><button id="inner">inner</button>`,
    )
    const outer = document.getElementById('outer') as HTMLButtonElement
    const inner = document.getElementById('inner') as HTMLButtonElement
    outer.focus()
    const fr = new FocusReturn()
    fr.save()
    inner.focus()
    fr.restore()
    expect(document.activeElement).toBe(outer)
  })
})
