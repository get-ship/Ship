// Tests for the smaller presentational components — spinner,
// separator, skeleton, alert. Each is small enough that one file
// covers all four reasonably.

import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-spinner (sail)', () => {
  let tag: string
  beforeAll(async () => {
    tag = await loadSailComponent('ship-spinner.set')
  }, 120_000)

  it('renders a .spin element with role=status and aria-label', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    const spin = el.querySelector('.spin')!
    expect(spin).not.toBeNull()
    expect(spin.getAttribute('role')).toBe('status')
    expect(spin.getAttribute('aria-label')).toBe('Loading')
    el.remove()
  })

  it('label prop overrides default', () => {
    const el = document.createElement(tag)
    el.setAttribute('label', 'Saving')
    document.body.appendChild(el)
    expect(el.querySelector('.spin')!.getAttribute('aria-label')).toBe('Saving')
    el.remove()
  })

  it('tone × size matrix renders without error', () => {
    const tones = ['default', 'info', 'success', 'warning', 'destructive', 'accent', 'muted']
    const sizes = ['sm', 'md', 'lg']
    for (const tone of tones) {
      for (const size of sizes) {
        const el = document.createElement(tag)
        el.setAttribute('tone', tone)
        el.setAttribute('size', size)
        document.body.appendChild(el)
        expect(el.querySelector('.spin')).not.toBeNull()
        el.remove()
      }
    }
  })
})

describe('ship-separator (sail)', () => {
  let tag: string
  beforeAll(async () => {
    tag = await loadSailComponent('ship-separator.set')
  }, 120_000)

  it('mounts with role=separator and default orientation=horizontal', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(el.getAttribute('role')).toBe('separator')
    expect(el.getAttribute('aria-orientation')).toBe('horizontal')
    el.remove()
  })

  it('orientation=vertical reflects to aria-orientation', () => {
    const el = document.createElement(tag)
    el.setAttribute('orientation', 'vertical')
    document.body.appendChild(el)
    expect(el.getAttribute('aria-orientation')).toBe('vertical')
    el.remove()
  })

  it('variant=dashed renders without error', () => {
    const el = document.createElement(tag)
    el.setAttribute('variant', 'dashed')
    document.body.appendChild(el)
    expect(el.hasAttribute('variant')).toBe(true)
    el.remove()
  })
})

describe('ship-skeleton (sail)', () => {
  let tag: string
  beforeAll(async () => {
    tag = await loadSailComponent('ship-skeleton.set')
  }, 120_000)

  it('renders a single .bar by default', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    const bars = el.querySelectorAll('.bar')
    expect(bars.length).toBe(1)
    el.remove()
  })

  it('lines=3 renders 3 bars (variant=text)', () => {
    const el = document.createElement(tag)
    el.setAttribute('lines', '3')
    document.body.appendChild(el)
    expect(el.querySelectorAll('.bar').length).toBe(3)
    el.remove()
  })

  it('variant=circle renders single bar regardless of lines', () => {
    const el = document.createElement(tag)
    el.setAttribute('variant', 'circle')
    el.setAttribute('lines', '5')
    document.body.appendChild(el)
    expect(el.querySelectorAll('.bar').length).toBe(1)
    el.remove()
  })

  it('width and height props override defaults', () => {
    const el = document.createElement(tag)
    el.setAttribute('width', '50%')
    el.setAttribute('height', '24px')
    document.body.appendChild(el)
    const bar = el.querySelector('.bar') as HTMLElement
    expect(bar.style.width).toBe('50%')
    expect(bar.style.height).toBe('24px')
    el.remove()
  })

  it('sets accessibility attributes on mount', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(el.getAttribute('role')).toBe('status')
    expect(el.getAttribute('aria-busy')).toBe('true')
    expect(el.getAttribute('aria-label')).toBe('Loading')
    el.remove()
  })
})

describe('ship-alert (sail)', () => {
  let tag: string
  beforeAll(async () => {
    tag = await loadSailComponent('ship-alert.set')
  }, 120_000)

  it('mounts with role=status and aria-live=polite by default (info)', () => {
    const el = document.createElement(tag)
    el.appendChild(document.createTextNode('Hello'))
    document.body.appendChild(el)
    const box = el.querySelector('.box')!
    expect(box.getAttribute('role')).toBe('status')
    expect(box.getAttribute('aria-live')).toBe('polite')
    el.remove()
  })

  it('destructive tone uses role=alert + aria-live=assertive', () => {
    const el = document.createElement(tag)
    el.setAttribute('tone', 'destructive')
    document.body.appendChild(el)
    const box = el.querySelector('.box')!
    expect(box.getAttribute('role')).toBe('alert')
    expect(box.getAttribute('aria-live')).toBe('assertive')
    el.remove()
  })

  it('warning tone is assertive', () => {
    const el = document.createElement(tag)
    el.setAttribute('tone', 'warning')
    document.body.appendChild(el)
    expect(el.querySelector('.box')!.getAttribute('aria-live')).toBe('assertive')
    el.remove()
  })

  it('dismissible renders close button; click fires ship-dismiss + removes host', () => {
    const el = document.createElement(tag)
    el.setAttribute('dismissible', '')
    el.appendChild(document.createTextNode('Hi'))
    document.body.appendChild(el)
    const closeBtn = el.querySelector('.close') as HTMLButtonElement
    expect(closeBtn).not.toBeNull()

    let fired = 0
    document.body.addEventListener('ship-dismiss', () => { fired++ }, { once: true })
    closeBtn.click()
    expect(fired).toBe(1)
    expect(document.body.contains(el)).toBe(false)
  })

  it('non-dismissible has no close button', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(el.querySelector('.close')).toBeNull()
    el.remove()
  })

  it('renders title/body/actions slots', () => {
    const el = document.createElement(tag)
    const title = document.createElement('span')
    title.setAttribute('slot', 'title')
    title.textContent = 'Heads up'
    const action = document.createElement('button')
    action.setAttribute('slot', 'actions')
    action.textContent = 'Retry'
    el.appendChild(title)
    el.appendChild(document.createTextNode('Body content'))
    el.appendChild(action)
    document.body.appendChild(el)

    expect(el.querySelector('.title')!.textContent).toContain('Heads up')
    expect(el.querySelector('.body')!.textContent).toContain('Body content')
    expect(el.querySelector('.actions')!.textContent).toContain('Retry')
    el.remove()
  })
})
