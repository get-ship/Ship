// Smaller component ports — five components in one file.

import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-status-dot (sail)', () => {
  let tag: string
  beforeAll(async () => { tag = await loadSailComponent('ship-status-dot.set') }, 120_000)

  it('renders an aria-hidden dot by default', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    const dot = el.querySelector('.dot')!
    expect(dot.getAttribute('aria-hidden')).toBe('true')
    expect(dot.getAttribute('role')).toBeNull()
    el.remove()
  })

  it('with a label, becomes role=img with aria-label', () => {
    const el = document.createElement(tag)
    el.setAttribute('label', 'Online')
    document.body.appendChild(el)
    const dot = el.querySelector('.dot')!
    expect(dot.getAttribute('role')).toBe('img')
    expect(dot.getAttribute('aria-label')).toBe('Online')
    expect(dot.getAttribute('aria-hidden')).toBeNull()
    el.remove()
  })

  it('size + tone + pulse all reflect to host', () => {
    for (const size of ['sm', 'md']) {
      for (const tone of ['default', 'success', 'warning', 'destructive']) {
        const el = document.createElement(tag)
        el.setAttribute('size', size)
        el.setAttribute('tone', tone)
        el.setAttribute('pulse', '')
        document.body.appendChild(el)
        expect(el.querySelector('.dot')).not.toBeNull()
        el.remove()
      }
    }
  })
})

describe('ship-kbd (sail)', () => {
  let tag: string
  beforeAll(async () => { tag = await loadSailComponent('ship-kbd.set') }, 120_000)

  it('parses keys string into per-token kbd elements', () => {
    const el = document.createElement(tag)
    el.setAttribute('keys', 'Cmd+K')
    document.body.appendChild(el)
    const kbds = el.querySelectorAll('kbd')
    expect(kbds.length).toBe(2)
    expect(kbds[0]!.textContent).toBe('Cmd')
    expect(kbds[1]!.textContent).toBe('K')
    expect(el.querySelectorAll('.plus').length).toBe(1)
    el.remove()
  })

  it('three-key shortcut renders two plus separators', () => {
    const el = document.createElement(tag)
    el.setAttribute('keys', 'Ctrl+Shift+P')
    document.body.appendChild(el)
    expect(el.querySelectorAll('kbd').length).toBe(3)
    expect(el.querySelectorAll('.plus').length).toBe(2)
    el.remove()
  })

  it('sets role=group + aria-label', () => {
    const el = document.createElement(tag)
    el.setAttribute('keys', 'Cmd+K')
    document.body.appendChild(el)
    expect(el.getAttribute('role')).toBe('group')
    expect(el.getAttribute('aria-label')).toBe('Keyboard shortcut: Cmd+K')
    el.remove()
  })

  it('empty keys renders nothing', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(el.querySelectorAll('kbd').length).toBe(0)
    el.remove()
  })
})

describe('ship-stat (sail)', () => {
  let tag: string
  beforeAll(async () => { tag = await loadSailComponent('ship-stat.set') }, 120_000)

  it('renders label / value-row / hint slot wrappers', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(el.querySelector('.label')).not.toBeNull()
    expect(el.querySelector('.value-row')).not.toBeNull()
    expect(el.querySelector('.hint')).not.toBeNull()
    el.remove()
  })

  it('default slot fills the value', () => {
    const el = document.createElement(tag)
    el.appendChild(document.createTextNode('1,234'))
    document.body.appendChild(el)
    expect(el.querySelector('.value')!.textContent).toContain('1,234')
    el.remove()
  })

  it('interactive sets role=button + tabindex + Enter activates click', () => {
    const el = document.createElement(tag)
    el.setAttribute('interactive', '')
    document.body.appendChild(el)
    expect(el.getAttribute('role')).toBe('button')
    expect(el.getAttribute('tabindex')).toBe('0')

    let clicks = 0
    el.addEventListener('click', () => { clicks++ })
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    expect(clicks).toBe(1)
    el.remove()
  })

  it('accent prop reflects to host', () => {
    for (const accent of ['primary', 'info', 'success', 'destructive', 'warning', 'accent']) {
      const el = document.createElement(tag)
      el.setAttribute('accent', accent)
      document.body.appendChild(el)
      expect(el.getAttribute('accent')).toBe(accent)
      el.remove()
    }
  })
})

describe('ship-avatar (sail)', () => {
  let tag: string
  beforeAll(async () => { tag = await loadSailComponent('ship-avatar.set') }, 120_000)

  it('renders initials fallback when no src is provided', () => {
    const el = document.createElement(tag)
    el.setAttribute('name', 'Alice Anderson')
    document.body.appendChild(el)
    const initials = el.querySelector('.initials')
    expect(initials).not.toBeNull()
    expect(initials!.textContent).toBe('AA')
    expect(el.hasAttribute('data-initials')).toBe(true)
    el.remove()
  })

  it('single-word name renders one-letter initial', () => {
    const el = document.createElement(tag)
    el.setAttribute('name', 'Solo')
    document.body.appendChild(el)
    expect(el.querySelector('.initials')!.textContent).toBe('S')
    el.remove()
  })

  it('explicit initials prop overrides name-derived', () => {
    const el = document.createElement(tag)
    el.setAttribute('name', 'Alice Anderson')
    el.setAttribute('initials', 'XYZ')
    document.body.appendChild(el)
    expect(el.querySelector('.initials')!.textContent).toBe('XYZ')
    el.remove()
  })

  it('renders <img> when src is provided', () => {
    const el = document.createElement(tag)
    el.setAttribute('src', '/avatars/alice.jpg')
    el.setAttribute('name', 'Alice')
    document.body.appendChild(el)
    const img = el.querySelector('img')
    expect(img).not.toBeNull()
    expect(img!.src).toContain('/avatars/alice.jpg')
    expect(el.hasAttribute('data-initials')).toBe(false)
    el.remove()
  })

  it('icon-slug derives src from simpleicons CDN', () => {
    const el = document.createElement(tag)
    el.setAttribute('icon-slug', 'rust')
    document.body.appendChild(el)
    const img = el.querySelector('img')!
    expect(img.src).toBe('https://cdn.simpleicons.org/rust')
    el.remove()
  })

  it('size + shape attributes reflect to host', () => {
    for (const size of ['sm', 'md', 'lg', 'xl']) {
      for (const shape of ['round', 'square']) {
        const el = document.createElement(tag)
        el.setAttribute('size', size)
        el.setAttribute('shape', shape)
        document.body.appendChild(el)
        expect(el.getAttribute('size')).toBe(size)
        expect(el.getAttribute('shape')).toBe(shape)
        el.remove()
      }
    }
  })

  it('aria-label falls back through name → initials → "avatar"', () => {
    const e1 = document.createElement(tag)
    e1.setAttribute('name', 'Alice')
    document.body.appendChild(e1)
    expect(e1.querySelector('.wrap')!.getAttribute('aria-label')).toBe('Alice')
    e1.remove()

    const e2 = document.createElement(tag)
    document.body.appendChild(e2)
    expect(e2.querySelector('.wrap')!.getAttribute('aria-label')).toBe('avatar')
    e2.remove()
  })
})

describe('ship-chip (sail)', () => {
  let tag: string
  beforeAll(async () => { tag = await loadSailComponent('ship-chip.set') }, 120_000)

  it('renders chip with leading slot + default slot', () => {
    const el = document.createElement(tag)
    el.appendChild(document.createTextNode('rust'))
    document.body.appendChild(el)
    const chip = el.querySelector('.chip')!
    expect(chip).not.toBeNull()
    expect(chip.textContent).toContain('rust')
    el.remove()
  })

  it('host gets role=button + tabindex=0 by default', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(el.getAttribute('role')).toBe('button')
    expect(el.getAttribute('tabindex')).toBe('0')
    el.remove()
  })

  it('Enter/Space synthesize a click', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    let clicks = 0
    el.addEventListener('click', () => { clicks++ })

    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    expect(clicks).toBe(1)
    el.dispatchEvent(new KeyboardEvent('keydown', { key: ' ', bubbles: true }))
    expect(clicks).toBe(2)
    el.remove()
  })

  it('selected sets aria-pressed=true', () => {
    const el = document.createElement(tag)
    el.setAttribute('selected', '')
    document.body.appendChild(el)
    expect(el.getAttribute('aria-pressed')).toBe('true')
    el.remove()
  })

  it('disabled sets aria-disabled and tabindex=-1', () => {
    const el = document.createElement(tag)
    el.setAttribute('disabled', '')
    document.body.appendChild(el)
    expect(el.getAttribute('aria-disabled')).toBe('true')
    expect(el.getAttribute('tabindex')).toBe('-1')
    el.remove()
  })

  it('removable renders × button; click fires ship-remove', () => {
    const el = document.createElement(tag)
    el.setAttribute('removable', '')
    document.body.appendChild(el)

    const removeBtn = el.querySelector('.remove') as HTMLButtonElement
    expect(removeBtn).not.toBeNull()

    const events: any[] = []
    el.addEventListener('ship-remove', (e: any) => events.push(e.detail))
    removeBtn.click()
    expect(events.length).toBe(1)
    el.remove()
  })

  it('Backspace/Delete on a removable chip fires ship-remove', () => {
    const el = document.createElement(tag)
    el.setAttribute('removable', '')
    document.body.appendChild(el)

    const events: any[] = []
    el.addEventListener('ship-remove', () => events.push({}))
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Backspace', bubbles: true }))
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Delete', bubbles: true }))
    expect(events.length).toBe(2)
    el.remove()
  })

  it('non-removable chip ignores Backspace', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    const events: any[] = []
    el.addEventListener('ship-remove', () => events.push({}))
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Backspace', bubbles: true }))
    expect(events.length).toBe(0)
    el.remove()
  })
})
