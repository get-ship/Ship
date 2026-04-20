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
})
