import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-segmented (sail)', () => {
  beforeAll(async () => {
    // Child first so the parent sees upgraded <ship-segmented-option> at mount.
    await loadSailComponent('ship-segmented-option.set')
    await loadSailComponent('ship-segmented.set')
  }, 120_000)

  function makeSegmented(attrs = ''): HTMLElement {
    const wrap = document.createElement('div')
    wrap.innerHTML = `
      <ship-segmented ${attrs}>
        <ship-segmented-option value="rows">Rows</ship-segmented-option>
        <ship-segmented-option value="schema">Schema</ship-segmented-option>
        <ship-segmented-option value="disabled-opt" disabled>Disabled</ship-segmented-option>
      </ship-segmented>
    `
    document.body.appendChild(wrap)
    return wrap.querySelector('ship-segmented') as HTMLElement
  }

  it('renders one button per option with role=radio', () => {
    const el = makeSegmented('value="rows"')
    const btns = el.querySelectorAll('button[role="radio"]')
    expect(btns.length).toBe(3)
    el.parentElement!.remove()
  })

  it('host has role=radiogroup', () => {
    const el = makeSegmented('value="rows"')
    expect(el.getAttribute('role')).toBe('radiogroup')
    el.parentElement!.remove()
  })

  it('aria-checked reflects selected value', () => {
    const el = makeSegmented('value="rows"')
    const btns = el.querySelectorAll('button[role="radio"]')
    expect(btns[0]!.getAttribute('aria-checked')).toBe('true')
    expect(btns[1]!.getAttribute('aria-checked')).toBe('false')
    expect(btns[2]!.getAttribute('aria-checked')).toBe('false')
    el.parentElement!.remove()
  })

  it('roving tabindex: only the selected option is tabindex=0', () => {
    const el = makeSegmented('value="rows"')
    const btns = el.querySelectorAll('button[role="radio"]')
    expect(btns[0]!.getAttribute('tabindex')).toBe('0')
    expect(btns[1]!.getAttribute('tabindex')).toBe('-1')
    expect(btns[2]!.getAttribute('tabindex')).toBe('-1')
    el.parentElement!.remove()
  })

  it('option label comes from textContent', () => {
    const el = makeSegmented('value="rows"')
    const btns = el.querySelectorAll('button[role="radio"]')
    expect(btns[0]!.querySelector('.btn-label')!.textContent).toBe('Rows')
    expect(btns[1]!.querySelector('.btn-label')!.textContent).toBe('Schema')
    el.parentElement!.remove()
  })

  it('clicking an option updates value and fires ship-change', () => {
    const el = makeSegmented('value="rows"')
    const events: any[] = []
    el.addEventListener('ship-change', (e: any) => events.push(e.detail))
    const btns = el.querySelectorAll('button[role="radio"]')
    ;(btns[1] as HTMLButtonElement).click()
    expect(el.getAttribute('value')).toBe('schema')
    expect(events).toEqual([{ value: 'schema' }])
    // aria-checked moved across.
    expect(btns[0]!.getAttribute('aria-checked')).toBe('false')
    expect(btns[1]!.getAttribute('aria-checked')).toBe('true')
    el.parentElement!.remove()
  })

  it('clicking the already-selected option does not re-fire ship-change', () => {
    const el = makeSegmented('value="rows"')
    const events: any[] = []
    el.addEventListener('ship-change', (e: any) => events.push(e.detail))
    const btns = el.querySelectorAll('button[role="radio"]')
    ;(btns[0] as HTMLButtonElement).click()
    expect(events).toEqual([])
    el.parentElement!.remove()
  })

  it('disabled option sets aria-disabled and blocks selection on click', () => {
    const el = makeSegmented('value="rows"')
    const events: any[] = []
    el.addEventListener('ship-change', (e: any) => events.push(e.detail))
    const btns = el.querySelectorAll('button[role="radio"]')
    const disabledBtn = btns[2] as HTMLButtonElement
    expect(disabledBtn.getAttribute('aria-disabled')).toBe('true')
    expect(disabledBtn.disabled).toBe(true)
    disabledBtn.click()
    expect(events).toEqual([])
    expect(el.getAttribute('value')).toBe('rows')
    el.parentElement!.remove()
  })

  it('host disabled blocks all options', () => {
    const wrap = document.createElement('div')
    wrap.innerHTML = `
      <ship-segmented value="rows" disabled>
        <ship-segmented-option value="rows">Rows</ship-segmented-option>
        <ship-segmented-option value="schema">Schema</ship-segmented-option>
      </ship-segmented>
    `
    document.body.appendChild(wrap)
    const el = wrap.querySelector('ship-segmented') as HTMLElement
    const events: any[] = []
    el.addEventListener('ship-change', (e: any) => events.push(e.detail))
    const btns = el.querySelectorAll('button[role="radio"]')
    expect((btns[0] as HTMLButtonElement).disabled).toBe(true)
    expect((btns[1] as HTMLButtonElement).disabled).toBe(true)
    ;(btns[1] as HTMLButtonElement).click()
    expect(events).toEqual([])
    expect(el.getAttribute('value')).toBe('rows')
    wrap.remove()
  })

  it('ArrowRight moves selection to next enabled option', () => {
    const el = makeSegmented('value="rows"')
    const btns = el.querySelectorAll('button[role="radio"]')
    const first = btns[0] as HTMLButtonElement
    first.dispatchEvent(new KeyboardEvent('keydown', {
      key: 'ArrowRight', bubbles: true, cancelable: true,
    }))
    expect(el.getAttribute('value')).toBe('schema')
    el.parentElement!.remove()
  })

  it('ArrowDown also advances (vertical/horizontal-agnostic)', () => {
    const el = makeSegmented('value="rows"')
    const first = el.querySelectorAll('button[role="radio"]')[0] as HTMLButtonElement
    first.dispatchEvent(new KeyboardEvent('keydown', {
      key: 'ArrowDown', bubbles: true, cancelable: true,
    }))
    expect(el.getAttribute('value')).toBe('schema')
    el.parentElement!.remove()
  })

  it('ArrowLeft from first wraps to last enabled option (skips disabled)', () => {
    const el = makeSegmented('value="rows"')
    const first = el.querySelectorAll('button[role="radio"]')[0] as HTMLButtonElement
    first.dispatchEvent(new KeyboardEvent('keydown', {
      key: 'ArrowLeft', bubbles: true, cancelable: true,
    }))
    // disabled option is filtered out, so wrap lands on 'schema'.
    expect(el.getAttribute('value')).toBe('schema')
    el.parentElement!.remove()
  })

  it('ArrowRight from last enabled wraps to first', () => {
    const el = makeSegmented('value="schema"')
    const second = el.querySelectorAll('button[role="radio"]')[1] as HTMLButtonElement
    second.dispatchEvent(new KeyboardEvent('keydown', {
      key: 'ArrowRight', bubbles: true, cancelable: true,
    }))
    expect(el.getAttribute('value')).toBe('rows')
    el.parentElement!.remove()
  })

  it('Enter activates the focused button', () => {
    const el = makeSegmented('value="rows"')
    const btns = el.querySelectorAll('button[role="radio"]')
    const second = btns[1] as HTMLButtonElement
    second.dispatchEvent(new KeyboardEvent('keydown', {
      key: 'Enter', bubbles: true, cancelable: true,
    }))
    expect(el.getAttribute('value')).toBe('schema')
    el.parentElement!.remove()
  })

  it('Space activates the focused button', () => {
    const el = makeSegmented('value="rows"')
    const second = el.querySelectorAll('button[role="radio"]')[1] as HTMLButtonElement
    second.dispatchEvent(new KeyboardEvent('keydown', {
      key: ' ', bubbles: true, cancelable: true,
    }))
    expect(el.getAttribute('value')).toBe('schema')
    el.parentElement!.remove()
  })

  it('non-navigation keys are ignored', () => {
    const el = makeSegmented('value="rows"')
    const events: any[] = []
    el.addEventListener('ship-change', (e: any) => events.push(e.detail))
    const first = el.querySelectorAll('button[role="radio"]')[0] as HTMLButtonElement
    first.dispatchEvent(new KeyboardEvent('keydown', {
      key: 'a', bubbles: true, cancelable: true,
    }))
    expect(events).toEqual([])
    expect(el.getAttribute('value')).toBe('rows')
    el.parentElement!.remove()
  })

  it('size attribute reflects to host', () => {
    const wrap = document.createElement('div')
    wrap.innerHTML = `
      <ship-segmented value="a" size="sm">
        <ship-segmented-option value="a">A</ship-segmented-option>
        <ship-segmented-option value="b">B</ship-segmented-option>
      </ship-segmented>
    `
    document.body.appendChild(wrap)
    const el = wrap.querySelector('ship-segmented') as HTMLElement
    expect(el.getAttribute('size')).toBe('sm')
    expect(el.querySelectorAll('button[role="radio"]').length).toBe(2)
    wrap.remove()
  })

  it('renders a container with part="container"', () => {
    const el = makeSegmented('value="rows"')
    const container = el.querySelector('.container')
    expect(container).not.toBeNull()
    expect(container!.getAttribute('part')).toBe('container')
    el.parentElement!.remove()
  })
})
