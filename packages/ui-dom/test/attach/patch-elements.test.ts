import { afterEach, describe, expect, it, vi } from 'vitest'
import {
  applyPatchElements,
  isValidSelector,
  type PatchElementsPayload,
} from '../../src/attach/patch-elements'
import { clearContainers, mkContainer } from './helpers'

afterEach(() => {
  clearContainers()
  vi.restoreAllMocks()
})

function viewRoot(inner: string): HTMLElement {
  const root = mkContainer()
  root.setAttribute('data-sui-root', '')
  root.setAttribute('data-view', 'demo')
  root.innerHTML = inner
  return root
}

describe('patch-elements/selector validation', () => {
  it('accepts ordinary CSS selectors', () => {
    expect(isValidSelector('#x')).toBe(true)
    expect(isValidSelector('.foo .bar')).toBe(true)
    expect(isValidSelector('[data-id="42"]')).toBe(true)
    expect(isValidSelector("button[type='submit']")).toBe(true)
  })

  it('rejects empty / non-string selectors', () => {
    expect(isValidSelector('')).toBe(false)
    expect(isValidSelector('   ')).toBe(false)
    expect(isValidSelector(null)).toBe(false)
    expect(isValidSelector(undefined)).toBe(false)
    expect(isValidSelector(42)).toBe(false)
  })

  it('rejects selectors that look like HTML', () => {
    expect(isValidSelector('<div>')).toBe(false)
    expect(isValidSelector('div > span<x')).toBe(false)
  })
})

describe('patch-elements/apply by mode', () => {
  it('replace swaps outerHTML', () => {
    const root = viewRoot('<div id="t" class="old">old</div>')
    const ok = applyPatchElements(root, {
      selector: '#t',
      mode: 'replace',
      html: '<section id="t" class="new">new</section>',
    })
    expect(ok).toBe(true)
    expect(root.innerHTML.trim()).toBe('<section id="t" class="new">new</section>')
  })

  it('append adds children at end', () => {
    const root = viewRoot('<ul id="t"><li>a</li></ul>')
    const ok = applyPatchElements(root, {
      selector: '#t',
      mode: 'append',
      html: '<li>b</li>',
    })
    expect(ok).toBe(true)
    const items = Array.from(root.querySelectorAll('#t li')).map((n) => n.textContent)
    expect(items).toEqual(['a', 'b'])
  })

  it('prepend adds children at start', () => {
    const root = viewRoot('<ul id="t"><li>z</li></ul>')
    applyPatchElements(root, {
      selector: '#t',
      mode: 'prepend',
      html: '<li>a</li>',
    })
    const items = Array.from(root.querySelectorAll('#t li')).map((n) => n.textContent)
    expect(items).toEqual(['a', 'z'])
  })

  it('before inserts as previous sibling', () => {
    const root = viewRoot('<div id="parent"><span id="t">x</span></div>')
    applyPatchElements(root, {
      selector: '#t',
      mode: 'before',
      html: '<i id="prev">!</i>',
    })
    const parent = root.querySelector('#parent')!
    expect(parent.firstElementChild!.id).toBe('prev')
    expect(parent.children[1]!.id).toBe('t')
  })

  it('after inserts as next sibling', () => {
    const root = viewRoot('<div id="parent"><span id="t">x</span></div>')
    applyPatchElements(root, {
      selector: '#t',
      mode: 'after',
      html: '<i id="next">!</i>',
    })
    const parent = root.querySelector('#parent')!
    expect(parent.children[0]!.id).toBe('t')
    expect(parent.children[1]!.id).toBe('next')
  })

  it('morph preserves element identity for matched tags', () => {
    const root = viewRoot('<form id="t"><input id="email" value="a@b"></form>')
    const formBefore = root.querySelector('#t')!
    const inputBefore = root.querySelector('#email')! as HTMLInputElement

    applyPatchElements(root, {
      selector: '#t',
      mode: 'morph',
      html: '<form id="t" class="ok"><input id="email" value="a@b" placeholder="email"></form>',
    })

    const formAfter = root.querySelector('#t')!
    const inputAfter = root.querySelector('#email')! as HTMLInputElement
    expect(formAfter).toBe(formBefore)
    expect(inputAfter).toBe(inputBefore)
    expect(formAfter.getAttribute('class')).toBe('ok')
    expect(inputAfter.getAttribute('placeholder')).toBe('email')
  })

  it('morph updates text without rebuilding the wrapping element', () => {
    const root = viewRoot('<p id="t">old</p>')
    const before = root.querySelector('#t')!
    applyPatchElements(root, {
      selector: '#t',
      mode: 'morph',
      html: '<p id="t">new</p>',
    })
    const after = root.querySelector('#t')!
    expect(after).toBe(before)
    expect(after.textContent).toBe('new')
  })

  it('morph removes excess children and inserts new keyed ones', () => {
    const root = viewRoot(
      '<ul id="t"><li data-key="a">A</li><li data-key="b">B</li></ul>',
    )
    applyPatchElements(root, {
      selector: '#t',
      mode: 'morph',
      html:
        '<ul id="t"><li data-key="b">B</li><li data-key="c">C</li></ul>',
    })
    const items = Array.from(root.querySelectorAll('#t li')).map((n) => ({
      key: (n as HTMLElement).dataset.key,
      text: n.textContent,
    }))
    expect(items).toEqual([
      { key: 'b', text: 'B' },
      { key: 'c', text: 'C' },
    ])
  })
})

describe('patch-elements/security boundary', () => {
  it('selectors are scoped to the view root, never document', () => {
    const root = viewRoot('<div id="inside">inside</div>')
    // Add an element OUTSIDE the view root with the same selector.
    const outside = document.createElement('div')
    outside.id = 'outside-target'
    outside.textContent = 'do-not-touch'
    document.body.appendChild(outside)

    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    const ok = applyPatchElements(root, {
      selector: '#outside-target',
      mode: 'replace',
      html: '<div id="outside-target">PWNED</div>',
    })
    expect(ok).toBe(false)
    expect(outside.textContent).toBe('do-not-touch')
    expect(warn).toHaveBeenCalled()
    outside.remove()
  })

  it('drops patches with empty selector and warns', () => {
    const root = viewRoot('<p id="t">x</p>')
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    const ok = applyPatchElements(root, {
      selector: '',
      mode: 'replace',
      html: '<p>y</p>',
    } as PatchElementsPayload)
    expect(ok).toBe(false)
    expect(root.querySelector('#t')!.textContent).toBe('x')
    expect(warn).toHaveBeenCalled()
  })

  it('drops patches with unknown mode and warns', () => {
    const root = viewRoot('<p id="t">x</p>')
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    const ok = applyPatchElements(root, {
      selector: '#t',
      mode: 'explode' as PatchElementsPayload['mode'],
      html: '<p>y</p>',
    })
    expect(ok).toBe(false)
    expect(root.querySelector('#t')!.textContent).toBe('x')
    expect(warn).toHaveBeenCalled()
  })

  it('drops patches with selector containing < and warns', () => {
    const root = viewRoot('<p id="t">x</p>')
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    const ok = applyPatchElements(root, {
      selector: '<script>',
      mode: 'replace',
      html: '<p>y</p>',
    })
    expect(ok).toBe(false)
    expect(warn).toHaveBeenCalled()
  })

  it('drops patches whose selector parses but matches nothing', () => {
    const root = viewRoot('<p id="t">x</p>')
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    const ok = applyPatchElements(root, {
      selector: '#never',
      mode: 'replace',
      html: '<p>y</p>',
    })
    expect(ok).toBe(false)
    expect(warn).toHaveBeenCalled()
  })
})

describe('patch-elements/settle', () => {
  it('toggles data-sui-settling for the requested duration', () => {
    vi.useFakeTimers()
    const root = viewRoot('<p id="t">x</p>')
    applyPatchElements(root, {
      selector: '#t',
      mode: 'morph',
      html: '<p id="t">y</p>',
      settle: 100,
    })
    const target = root.querySelector('#t')!
    expect(target.hasAttribute('data-sui-settling')).toBe(true)
    vi.advanceTimersByTime(101)
    expect(target.hasAttribute('data-sui-settling')).toBe(false)
    vi.useRealTimers()
  })

  it('omitting settle leaves no transition attribute', () => {
    const root = viewRoot('<p id="t">x</p>')
    applyPatchElements(root, {
      selector: '#t',
      mode: 'morph',
      html: '<p id="t">y</p>',
    })
    expect(root.querySelector('#t')!.hasAttribute('data-sui-settling')).toBe(false)
  })
})
