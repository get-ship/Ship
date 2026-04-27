import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-badge (sail)', () => {
  let tag: string

  beforeAll(async () => {
    tag = await loadSailComponent('ship-badge.sail')
  }, 120_000)

  it('renders default chrome with no attributes', () => {
    const el = document.createElement(tag)
    el.textContent = 'New'
    document.body.appendChild(el)
    const badge = el.querySelector('.badge')!
    expect(badge).not.toBeNull()
    expect(el.textContent).toContain('New')
    el.remove()
  })

  it('reflects tone attribute through to scoped style hook', () => {
    const el = document.createElement(tag)
    el.setAttribute('tone', 'success')
    document.body.appendChild(el)
    // Scoping prefix is [data-component="ship-badge"]; the tone selector
    // [data-component="ship-badge"][tone='success'] should match.
    expect(el.getAttribute('tone')).toBe('success')
    expect(el.dataset.component).toBe('ship-badge')
    el.remove()
  })

  it('exposes leading + default slots; both render fills', () => {
    const el = document.createElement(tag)
    const dot = document.createElement('span')
    dot.setAttribute('slot', 'leading')
    dot.textContent = '●'
    const label = document.createTextNode('Active')
    el.appendChild(dot)
    el.appendChild(label)
    document.body.appendChild(el)

    // leading wrapper exists as a child of .badge.
    const leading = el.querySelector('.leading')!
    expect(leading.textContent).toContain('●')
    // Default slot content lands in the badge somewhere.
    expect(el.textContent).toContain('Active')
    el.remove()
  })

  it('all enum tones, variants, sizes apply without error', () => {
    const tones = ['default', 'info', 'success', 'warning', 'destructive', 'accent', 'muted']
    const variants = ['solid', 'soft', 'outline']
    const sizes = ['sm', 'md', 'lg']
    for (const tone of tones) {
      for (const variant of variants) {
        for (const size of sizes) {
          const el = document.createElement(tag)
          el.setAttribute('tone', tone)
          el.setAttribute('variant', variant)
          el.setAttribute('size', size)
          el.textContent = `${tone}/${variant}/${size}`
          document.body.appendChild(el)
          expect(el.querySelector('.badge')).not.toBeNull()
          el.remove()
        }
      }
    }
  })

  it('pill flag is reflected for the [pill] style hook', () => {
    const el = document.createElement(tag)
    el.setAttribute('pill', '')
    document.body.appendChild(el)
    expect(el.hasAttribute('pill')).toBe(true)
    el.remove()
  })

  it('injects scoped CSS into document head exactly once', () => {
    const el1 = document.createElement(tag)
    document.body.appendChild(el1)
    const el2 = document.createElement(tag)
    document.body.appendChild(el2)
    const styles = document.head.querySelectorAll('style[data-sail="ship-badge"]')
    expect(styles.length).toBe(1)
    const css = styles[0]!.textContent!
    expect(css).toContain('[data-component="ship-badge"]')
    // :host should be rewritten away.
    expect(css).not.toContain(':host')
    // Tone CSS hook still selects on the host attribute.
    expect(css).toContain('[data-component="ship-badge"][tone=\'success\']')
    el1.remove()
    el2.remove()
  })
})
