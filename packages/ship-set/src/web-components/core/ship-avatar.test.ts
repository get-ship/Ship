import { afterEach, describe, expect, it } from 'vitest'
import './ship-avatar'
import { ShipAvatar, initialsFromName } from './ship-avatar'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipAvatar
  await el.updateComplete
  return el
}

describe('initialsFromName', () => {
  it('returns single uppercase char for one word', () => {
    expect(initialsFromName('ada')).toBe('A')
  })
  it('returns two chars for multi-word names', () => {
    expect(initialsFromName('ada lovelace')).toBe('AL')
    expect(initialsFromName('Ada Byron Lovelace')).toBe('AL')
  })
  it('returns empty string for blank names', () => {
    expect(initialsFromName('   ')).toBe('')
  })
})

describe('ship-avatar', () => {
  afterEach(cleanupTestDom)

  it('renders initials when no src', async () => {
    const el = await mount('<ship-avatar name="Ada Lovelace"></ship-avatar>')
    expect(el.shadowRoot!.textContent).toContain('AL')
  })

  it('renders image when src provided', async () => {
    const el = await mount('<ship-avatar src="/pic.png" name="Ada"></ship-avatar>')
    expect(el.shadowRoot!.querySelector('img')).not.toBeNull()
  })

  it('falls back to initials on image error', async () => {
    const el = await mount('<ship-avatar src="/bad.png" name="Ada"></ship-avatar>')
    const img = el.shadowRoot!.querySelector('img')!
    img.dispatchEvent(new Event('error'))
    await el.updateComplete
    expect(el.shadowRoot!.querySelector('img')).toBeNull()
    expect(el.shadowRoot!.textContent).toContain('A')
  })

  it('renders the explicit initials attribute and applies the rich tile treatment', async () => {
    const el = await mount('<ship-avatar initials="AB"></ship-avatar>')
    // Explicit initials render in the shadow DOM, uppercased.
    const node = el.shadowRoot!.querySelector('.initials') as HTMLElement
    expect(node).not.toBeNull()
    expect(node.textContent).toContain('AB')
    // Host carries the data-initials marker so CSS applies the gradient
    // treatment — the styling hook exists even if jsdom can't resolve
    // color-mix in computed style.
    expect(el.hasAttribute('data-initials')).toBe(true)
  })

  it('uppercases lowercase explicit initials', async () => {
    const el = await mount('<ship-avatar initials="ab"></ship-avatar>')
    const node = el.shadowRoot!.querySelector('.initials') as HTMLElement
    expect(node.textContent).toContain('AB')
  })

  it('drops the data-initials marker when an image is loaded', async () => {
    const el = await mount('<ship-avatar src="/pic.png" name="Ada"></ship-avatar>')
    expect(el.hasAttribute('data-initials')).toBe(false)
  })

  it('size="sm" is 28px to match compact list-tile rows', async () => {
    // The prior 24px sm was smaller than the pre-migration 28px and
    // had no production usage; bumping to 28px lets compact
    // list-tile rows carry the gradient tile without reaching for md.
    const staticStyles = ShipAvatar.styles as unknown as Array<{ cssText: string }>
    const css = staticStyles.map((s) => s.cssText || String(s)).join('\n')
    expect(css).toContain(`:host([size='sm']) { width: 28px; height: 28px;`)
    expect(css).not.toContain('width: 24px')
  })

  it('shape="square" uses --radius-sm instead of the round mask', async () => {
    const el = await mount('<ship-avatar shape="square" initials="C"></ship-avatar>')
    // The attribute reflects so CSS can target it; the style rule lives
    // in the static sheet.
    expect(el.getAttribute('shape')).toBe('square')
    const staticStyles = ShipAvatar.styles as unknown as Array<{ cssText: string }>
    const css = staticStyles.map((s) => s.cssText || String(s)).join('\n')
    expect(css).toContain(`:host([shape='square']) { border-radius: var(--radius-sm); }`)
  })

  it('shape defaults to round when unset', async () => {
    const el = await mount('<ship-avatar initials="C"></ship-avatar>')
    // Default reflects as 'round'; the important invariant is that it's
    // not 'square' so the round border-radius rule wins.
    expect(el.shape).toBe('round')
    expect(el.getAttribute('shape')).not.toBe('square')
  })

  it('tone="primary" with initials uses --primary background, not the gradient', async () => {
    const el = await mount('<ship-avatar tone="primary" initials="C"></ship-avatar>')
    // Data marker is still present (no image loaded)...
    expect(el.hasAttribute('data-initials')).toBe(true)
    // ...and the style rule for the primary override exists.
    const staticStyles = ShipAvatar.styles as unknown as Array<{ cssText: string }>
    const css = staticStyles.map((s) => s.cssText || String(s)).join('\n')
    expect(css).toContain(
      `:host([data-initials][tone='primary']) {\n        background: var(--primary);\n        color: var(--primary-foreground);\n      }`,
    )
  })

  it('tone unset keeps the gradient on initials', async () => {
    const el = await mount('<ship-avatar initials="C"></ship-avatar>')
    expect(el.hasAttribute('data-initials')).toBe(true)
    // Tone defaults to 'muted' — the gradient [data-initials] rule wins
    // because there's no primary/accent/info/success/etc override for it.
    const staticStyles = ShipAvatar.styles as unknown as Array<{ cssText: string }>
    const css = staticStyles.map((s) => s.cssText || String(s)).join('\n')
    expect(css).toContain(':host([data-initials]) {')
    expect(css).toContain('linear-gradient(')
  })

  it('tone="accent" with initials uses solid accent background', async () => {
    const el = await mount('<ship-avatar tone="accent" initials="C"></ship-avatar>')
    expect(el.hasAttribute('data-initials')).toBe(true)
    const staticStyles = ShipAvatar.styles as unknown as Array<{ cssText: string }>
    const css = staticStyles.map((s) => s.cssText || String(s)).join('\n')
    expect(css).toContain(
      `:host([data-initials][tone='accent']) {\n        background: var(--accent);\n        color: var(--accent-foreground);\n      }`,
    )
  })

  it('icon-slug populates the image src from the simpleicons CDN', async () => {
    const el = await mount('<ship-avatar icon-slug="rust"></ship-avatar>')
    const img = el.shadowRoot!.querySelector('img')
    expect(img).not.toBeNull()
    expect(img!.getAttribute('src')).toBe('https://cdn.simpleicons.org/rust')
    // Image mode drops the initials marker.
    expect(el.hasAttribute('data-initials')).toBe(false)
  })

  it('explicit src wins over icon-slug', async () => {
    const el = await mount(
      '<ship-avatar src="/my.png" icon-slug="rust"></ship-avatar>',
    )
    const img = el.shadowRoot!.querySelector('img')!
    expect(img.getAttribute('src')).toBe('/my.png')
  })

  it('falls back to initials when the simpleicons image errors', async () => {
    const el = await mount(
      '<ship-avatar icon-slug="does-not-exist" initials="R"></ship-avatar>',
    )
    const img = el.shadowRoot!.querySelector('img')!
    img.dispatchEvent(new Event('error'))
    await el.updateComplete
    expect(el.shadowRoot!.querySelector('img')).toBeNull()
    expect(el.shadowRoot!.textContent).toContain('R')
    expect(el.hasAttribute('data-initials')).toBe(true)
  })
})
