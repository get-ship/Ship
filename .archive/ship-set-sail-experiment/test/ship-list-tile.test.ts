import { describe, expect, it, beforeAll, afterEach } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-list-tile (sail)', () => {
  let tag: string

  beforeAll(async () => {
    tag = await loadSailComponent('ship-list-tile.set')
  }, 120_000)

  afterEach(() => {
    // Tests append nodes with expectation that document.body is sole
    // owner; clear leaked refs so divider stylesheet checks stay
    // deterministic across cases.
    document.body.innerHTML = ''
  })

  it('renders a div tile with .row + .secondary structure', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    const tile = el.querySelector('.tile') as HTMLElement
    expect(tile).not.toBeNull()
    expect(tile.tagName).toBe('DIV')
    expect(el.querySelector('.row')).not.toBeNull()
    expect(el.querySelector('.primary')).not.toBeNull()
    expect(el.querySelector('.secondary')).not.toBeNull()
  })

  it('default role is listitem', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(el.getAttribute('role')).toBe('listitem')
  })

  it('reflects selected to aria-selected', () => {
    const el = document.createElement(tag)
    el.setAttribute('selected', '')
    document.body.appendChild(el)
    expect(el.getAttribute('aria-selected')).toBe('true')
  })

  it('reflects disabled to aria-disabled', () => {
    const el = document.createElement(tag)
    el.setAttribute('disabled', '')
    document.body.appendChild(el)
    expect(el.getAttribute('aria-disabled')).toBe('true')
  })

  it('sets role=button when interactive', () => {
    const el = document.createElement(tag)
    el.setAttribute('interactive', '')
    document.body.appendChild(el)
    expect(el.getAttribute('role')).toBe('button')
    const tile = el.querySelector('.tile') as HTMLElement
    expect(tile.getAttribute('tabindex')).toBe('0')
  })

  it('does not set tabindex when not interactive', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    const tile = el.querySelector('.tile') as HTMLElement
    expect(tile.getAttribute('tabindex')).toBeFalsy()
  })

  it('dispatches ship-select on click when interactive', () => {
    const el = document.createElement(tag)
    el.setAttribute('interactive', '')
    document.body.appendChild(el)
    let fired = 0
    el.addEventListener('ship-select', () => fired++)
    ;(el.querySelector('.tile') as HTMLElement).click()
    expect(fired).toBe(1)
  })

  it('does not dispatch ship-select when not interactive', () => {
    const el = document.createElement(tag)
    document.body.appendChild(el)
    let fired = 0
    el.addEventListener('ship-select', () => fired++)
    ;(el.querySelector('.tile') as HTMLElement).click()
    expect(fired).toBe(0)
  })

  it('dispatches ship-select on Enter and Space keys when interactive', () => {
    const el = document.createElement(tag)
    el.setAttribute('interactive', '')
    document.body.appendChild(el)
    let fired = 0
    el.addEventListener('ship-select', () => fired++)
    const tile = el.querySelector('.tile') as HTMLElement
    tile.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true }))
    tile.dispatchEvent(new KeyboardEvent('keydown', { key: ' ', bubbles: true, cancelable: true }))
    expect(fired).toBe(2)
  })

  it('does not dispatch ship-select on other keys', () => {
    const el = document.createElement(tag)
    el.setAttribute('interactive', '')
    document.body.appendChild(el)
    let fired = 0
    el.addEventListener('ship-select', () => fired++)
    const tile = el.querySelector('.tile') as HTMLElement
    tile.dispatchEvent(new KeyboardEvent('keydown', { key: 'a', bubbles: true, cancelable: true }))
    tile.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true, cancelable: true }))
    expect(fired).toBe(0)
  })

  it('blocks ship-select dispatch when disabled', () => {
    const el = document.createElement(tag)
    el.setAttribute('interactive', '')
    el.setAttribute('disabled', '')
    document.body.appendChild(el)
    let fired = 0
    el.addEventListener('ship-select', () => fired++)
    const tile = el.querySelector('.tile') as HTMLElement
    tile.click()
    tile.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true }))
    expect(fired).toBe(0)
    expect(el.getAttribute('aria-disabled')).toBe('true')
  })

  it('reflects variant and density attributes', () => {
    const el = document.createElement(tag)
    el.setAttribute('variant', 'card')
    el.setAttribute('density', 'compact')
    document.body.appendChild(el)
    expect(el.getAttribute('variant')).toBe('card')
    expect(el.getAttribute('density')).toBe('compact')
  })

  it('default slot renders primary content', () => {
    const el = document.createElement(tag)
    el.appendChild(document.createTextNode('Item label'))
    document.body.appendChild(el)
    expect(el.querySelector('.primary')!.textContent).toContain('Item label')
  })

  it('leading slot renders left of primary', () => {
    const el = document.createElement(tag)
    const lead = document.createElement('span')
    lead.setAttribute('slot', 'leading')
    lead.textContent = 'L'
    el.appendChild(lead)
    el.appendChild(document.createTextNode('Body'))
    document.body.appendChild(el)
    expect(el.querySelector('.row')!.textContent).toContain('L')
  })

  it('trailing slot renders to the right region', () => {
    const el = document.createElement(tag)
    const trail = document.createElement('span')
    trail.setAttribute('slot', 'trailing')
    trail.textContent = 'T'
    el.appendChild(document.createTextNode('Body'))
    el.appendChild(trail)
    document.body.appendChild(el)
    expect(el.querySelector('[part="trailing"]')!.textContent).toContain('T')
  })

  it('meta slot renders into the secondary region', () => {
    const el = document.createElement(tag)
    const meta = document.createElement('span')
    meta.setAttribute('slot', 'meta')
    meta.textContent = 'sub-label'
    el.appendChild(document.createTextNode('Primary'))
    el.appendChild(meta)
    document.body.appendChild(el)
    expect(el.querySelector('.secondary')!.textContent).toContain('sub-label')
  })

  describe('platform-native navigation', () => {
    // Navigation is the platform's job. Wrap the tile in <a href> when
    // you want a link and you get cmd-click, middle-click, prefetch,
    // view-transitions, and the right ARIA role for free. The tile
    // itself stays a presentational div.
    it('renders as div when wrapped in an anchor (caller owns navigation)', () => {
      const link = document.createElement('a')
      link.setAttribute('href', '/agents/foo')
      const el = document.createElement(tag)
      link.appendChild(el)
      document.body.appendChild(link)
      const tile = el.querySelector('.tile') as HTMLElement
      expect(tile.tagName).toBe('DIV')
      expect((link as HTMLAnchorElement).getAttribute('href')).toBe('/agents/foo')
    })
  })

  describe('inter-sibling dividers', () => {
    it('installs a document-level <style> on first connect', () => {
      const el = document.createElement(tag)
      el.setAttribute('variant', 'row')
      document.body.appendChild(el)
      const style = document.head.querySelector('style[data-ship-list-tile-dividers]')
      expect(style).not.toBeNull()
      expect(style!.textContent).toContain("ship-list-tile[variant='row']")
    })

    it('does not duplicate the divider stylesheet across instances', () => {
      const a = document.createElement(tag)
      const b = document.createElement(tag)
      document.body.appendChild(a)
      document.body.appendChild(b)
      const styles = document.head.querySelectorAll('style[data-ship-list-tile-dividers]')
      expect(styles.length).toBe(1)
    })
  })

  it('tone × variant × density matrix renders without error', () => {
    const tones = ['default', 'info', 'success', 'warning', 'destructive', 'accent', 'muted', 'primary']
    const variants = ['row', 'card']
    const densities = ['compact', 'normal', 'relaxed']
    let n = 0
    for (const tone of tones) {
      for (const variant of variants) {
        for (const density of densities) {
          const el = document.createElement(tag)
          el.setAttribute('tone', tone)
          el.setAttribute('variant', variant)
          el.setAttribute('density', density)
          document.body.appendChild(el)
          expect(el.querySelector('.tile')).not.toBeNull()
          el.remove()
          n++
        }
      }
    }
    expect(n).toBe(8 * 2 * 3)
  })
})
