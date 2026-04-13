import { afterEach, describe, expect, it } from 'vitest'
import './ship-avatar'
import { ShipAvatar, initialsFromName } from './ship-avatar'
import { cleanupTestDom } from '../util'

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
})
