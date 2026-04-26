import { describe, expect, it } from 'vitest'
import { signal } from '../src/signal'
import {
  bindText,
  bindShow,
  bindAttr,
  bindClass,
  bindProp,
  bindValue,
  bindChecked,
} from '../src/bind'

describe('bindText', () => {
  it('sets initial text', () => {
    const el = document.createElement('span')
    const $msg = signal('hello')
    bindText(el, () => $msg())
    expect(el.textContent).toBe('hello')
  })

  it('updates on signal change', () => {
    const el = document.createElement('span')
    const $msg = signal('a')
    bindText(el, () => $msg())
    $msg.set('b')
    expect(el.textContent).toBe('b')
  })

  it('null/undefined → empty', () => {
    const el = document.createElement('span')
    const $msg = signal<string | null>(null)
    bindText(el, () => $msg())
    expect(el.textContent).toBe('')
    $msg.set('x')
    expect(el.textContent).toBe('x')
    $msg.set(null)
    expect(el.textContent).toBe('')
  })

  it('disposer stops updates', () => {
    const el = document.createElement('span')
    const $msg = signal('a')
    const dispose = bindText(el, () => $msg())
    dispose()
    $msg.set('b')
    expect(el.textContent).toBe('a')
  })
})

describe('bindShow', () => {
  it('truthy → visible', () => {
    const el = document.createElement('div')
    const $on = signal(true)
    bindShow(el, () => $on())
    expect(el.style.display).toBe('')
  })

  it('falsy → display:none', () => {
    const el = document.createElement('div')
    const $on = signal(false)
    bindShow(el, () => $on())
    expect(el.style.display).toBe('none')
  })

  it('toggles on signal change', () => {
    const el = document.createElement('div')
    const $on = signal(true)
    bindShow(el, () => $on())
    $on.set(false)
    expect(el.style.display).toBe('none')
    $on.set(true)
    expect(el.style.display).toBe('')
  })

  it('preserves non-default display through hide/show', () => {
    const el = document.createElement('div')
    el.style.display = 'flex'
    const $on = signal(true)
    bindShow(el, () => $on())
    $on.set(false)
    expect(el.style.display).toBe('none')
    $on.set(true)
    expect(el.style.display).toBe('flex')
  })
})

describe('bindAttr', () => {
  it('sets string attribute', () => {
    const el = document.createElement('a')
    const $href = signal('/home')
    bindAttr(el, 'href', () => $href())
    expect(el.getAttribute('href')).toBe('/home')
    $href.set('/about')
    expect(el.getAttribute('href')).toBe('/about')
  })

  it('null removes attribute', () => {
    const el = document.createElement('button')
    const $disabled = signal<boolean | null>(null)
    bindAttr(el, 'disabled', () => $disabled())
    expect(el.hasAttribute('disabled')).toBe(false)
  })

  it('false removes attribute', () => {
    const el = document.createElement('button')
    el.setAttribute('disabled', '')
    const $disabled = signal(false)
    bindAttr(el, 'disabled', () => $disabled())
    expect(el.hasAttribute('disabled')).toBe(false)
  })

  it('true sets empty attribute (presence-only)', () => {
    const el = document.createElement('button')
    const $disabled = signal(true)
    bindAttr(el, 'disabled', () => $disabled())
    expect(el.getAttribute('disabled')).toBe('')
  })

  it('non-bool values stringify', () => {
    const el = document.createElement('div')
    const $count = signal(42)
    bindAttr(el, 'data-count', () => $count())
    expect(el.getAttribute('data-count')).toBe('42')
  })
})

describe('bindClass', () => {
  it('truthy adds class', () => {
    const el = document.createElement('div')
    const $active = signal(true)
    bindClass(el, 'active', () => $active())
    expect(el.classList.contains('active')).toBe(true)
  })

  it('toggles on change', () => {
    const el = document.createElement('div')
    const $active = signal(false)
    bindClass(el, 'active', () => $active())
    expect(el.classList.contains('active')).toBe(false)
    $active.set(true)
    expect(el.classList.contains('active')).toBe(true)
    $active.set(false)
    expect(el.classList.contains('active')).toBe(false)
  })

  it('does not interfere with other classes', () => {
    const el = document.createElement('div')
    el.className = 'base'
    const $active = signal(true)
    bindClass(el, 'active', () => $active())
    expect(el.classList.contains('base')).toBe(true)
    expect(el.classList.contains('active')).toBe(true)
  })
})

describe('bindProp', () => {
  it('sets a property directly', () => {
    const el = document.createElement('div') as HTMLDivElement & { items?: number[] }
    const $items = signal([1, 2, 3])
    bindProp(el, 'items', () => $items())
    expect(el.items).toEqual([1, 2, 3])
  })

  it('updates on signal change with reference fidelity', () => {
    const el = document.createElement('div') as HTMLDivElement & { data?: object }
    const $data = signal({ x: 1 })
    bindProp(el, 'data', () => $data())
    const next = { x: 2 }
    $data.set(next)
    expect(el.data).toBe(next) // same reference, not stringified
  })
})

describe('bindValue', () => {
  it('initial value flows signal → input', () => {
    const el = document.createElement('input')
    const $v = signal('hello')
    bindValue(el, $v)
    expect(el.value).toBe('hello')
  })

  it('signal change flows to input', () => {
    const el = document.createElement('input')
    const $v = signal('a')
    bindValue(el, $v)
    $v.set('b')
    expect(el.value).toBe('b')
  })

  it('user input flows back to signal', () => {
    const el = document.createElement('input')
    const $v = signal('')
    bindValue(el, $v)
    el.value = 'typed'
    el.dispatchEvent(new Event('input'))
    expect($v()).toBe('typed')
  })

  it('disposer detaches both directions', () => {
    const el = document.createElement('input')
    const $v = signal('a')
    const dispose = bindValue(el, $v)
    dispose()
    $v.set('b')
    expect(el.value).toBe('a') // signal→input detached
    el.value = 'c'
    el.dispatchEvent(new Event('input'))
    expect($v()).toBe('b') // input→signal detached; signal stays at 'b' from previous .set
  })
})

describe('bindChecked', () => {
  it('signal flows to checkbox', () => {
    const el = document.createElement('input')
    el.type = 'checkbox'
    const $on = signal(true)
    bindChecked(el, $on)
    expect(el.checked).toBe(true)
    $on.set(false)
    expect(el.checked).toBe(false)
  })

  it('checkbox change flows to signal', () => {
    const el = document.createElement('input')
    el.type = 'checkbox'
    const $on = signal(false)
    bindChecked(el, $on)
    el.checked = true
    el.dispatchEvent(new Event('change'))
    expect($on()).toBe(true)
  })
})
