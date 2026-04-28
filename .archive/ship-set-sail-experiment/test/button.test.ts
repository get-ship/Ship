import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-button (sail)', () => {
  let tag: string

  beforeAll(async () => {
    tag = await loadSailComponent('ship-button.set')
  }, 120_000)

  it('renders a native button with default chrome', () => {
    const el = document.createElement(tag)
    el.textContent = 'Save'
    document.body.appendChild(el)
    const btn = el.querySelector('button')!
    expect(btn).not.toBeNull()
    expect(btn.getAttribute('type')).toBe('button')
    expect(btn.disabled).toBe(false)
    expect(el.textContent).toContain('Save')
    el.remove()
  })

  it('disabled prop disables the underlying button', () => {
    const el = document.createElement(tag)
    el.setAttribute('disabled', '')
    document.body.appendChild(el)
    const btn = el.querySelector('button') as HTMLButtonElement
    expect(btn.disabled).toBe(true)
    expect(btn.getAttribute('aria-disabled')).toBe('true')
    el.remove()
  })

  it('loading shows a spinner and marks aria-busy', () => {
    const el = document.createElement(tag)
    el.setAttribute('loading', '')
    document.body.appendChild(el)
    const btn = el.querySelector('button') as HTMLButtonElement
    // loading also disables to prevent double-submits.
    expect(btn.disabled).toBe(true)
    expect(btn.getAttribute('aria-busy')).toBe('true')
    expect(el.querySelector('.spinner')).not.toBeNull()

    // Toggle off — spinner gone, button enabled.
    el.removeAttribute('loading')
    expect(btn.disabled).toBe(false)
    expect(btn.getAttribute('aria-busy')).toBe('false')
    expect(el.querySelector('.spinner')).toBeNull()
    el.remove()
  })

  it('type attribute flows to the native button', () => {
    const el = document.createElement(tag)
    el.setAttribute('type', 'submit')
    document.body.appendChild(el)
    expect(el.querySelector('button')!.getAttribute('type')).toBe('submit')
    el.remove()
  })

  it('label / pressed / expanded propagate as aria-* attributes', () => {
    const el = document.createElement(tag)
    el.setAttribute('label', 'Toggle menu')
    el.setAttribute('pressed', 'true')
    el.setAttribute('expanded', 'false')
    document.body.appendChild(el)
    const btn = el.querySelector('button')!
    expect(btn.getAttribute('aria-label')).toBe('Toggle menu')
    expect(btn.getAttribute('aria-pressed')).toBe('true')
    expect(btn.getAttribute('aria-expanded')).toBe('false')
    el.remove()
  })

  it('leading / trailing slots distribute icons around the label', () => {
    const el = document.createElement(tag)
    const lead = document.createElement('span')
    lead.setAttribute('slot', 'leading')
    lead.textContent = '◀'
    const trail = document.createElement('span')
    trail.setAttribute('slot', 'trailing')
    trail.textContent = '▶'
    el.appendChild(lead)
    el.appendChild(document.createTextNode('Continue'))
    el.appendChild(trail)
    document.body.appendChild(el)
    const text = el.querySelector('button')!.textContent!
    expect(text).toContain('◀')
    expect(text).toContain('Continue')
    expect(text).toContain('▶')
    el.remove()
  })

  it('native click bubbles out through the host', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    let count = 0
    el.addEventListener('click', () => { count++ })
    ;(el.querySelector('button') as HTMLButtonElement).click()
    expect(count).toBe(1)
    el.remove()
  })

  it('disabled blocks click events from firing', () => {
    const el = document.createElement(tag)
    el.setAttribute('disabled', '')
    document.body.appendChild(el)
    let count = 0
    el.addEventListener('click', () => { count++ })
    ;(el.querySelector('button') as HTMLButtonElement).click()
    // Native disabled buttons don't fire click; we delegate to the platform.
    expect(count).toBe(0)
    el.remove()
  })

  it('all variant × tone × size combinations render without error', () => {
    const tones = ['default', 'primary', 'info', 'success', 'warning', 'destructive', 'accent', 'muted']
    const variants = ['solid', 'soft', 'outline', 'ghost']
    const sizes = ['sm', 'md', 'lg']
    let n = 0
    for (const tone of tones) {
      for (const variant of variants) {
        for (const size of sizes) {
          const el = document.createElement(tag)
          el.setAttribute('tone', tone)
          el.setAttribute('variant', variant)
          el.setAttribute('size', size)
          el.textContent = 'go'
          document.body.appendChild(el)
          expect(el.querySelector('button')).not.toBeNull()
          el.remove()
          n++
        }
      }
    }
    expect(n).toBe(8 * 4 * 3) // 96 combinations
  })
})
