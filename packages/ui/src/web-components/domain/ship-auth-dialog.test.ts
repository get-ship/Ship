import { describe, it, expect, beforeEach, vi } from 'vitest'
import './ship-auth-dialog'
import type { ShipAuthDialog } from './ship-auth-dialog'

const tick = () => new Promise((r) => requestAnimationFrame(() => r(null)))

async function mount(html: string): Promise<ShipAuthDialog> {
  document.body.innerHTML = html
  const el = document.querySelector('ship-auth-dialog') as ShipAuthDialog
  await tick()
  await el.updateComplete
  return el
}

describe('<ship-auth-dialog>', () => {
  beforeEach(() => { document.body.innerHTML = '' })

  it('renders the sign-in title by default', async () => {
    const el = await mount('<ship-auth-dialog open></ship-auth-dialog>')
    const header = el.renderRoot.querySelector('[slot="header"]') as HTMLElement
    expect(header.textContent?.trim()).toContain('Welcome back')
  })

  it('renders the sign-up title when mode=sign-up', async () => {
    const el = await mount('<ship-auth-dialog open mode="sign-up"></ship-auth-dialog>')
    const header = el.renderRoot.querySelector('[slot="header"]') as HTMLElement
    expect(header.textContent?.trim()).toContain('Create your account')
  })

  it('switches mode and emits ship-mode-change when the inline button is clicked', async () => {
    const el = await mount('<ship-auth-dialog open></ship-auth-dialog>')
    const listener = vi.fn()
    el.addEventListener('ship-mode-change', listener)
    const btn = el.renderRoot.querySelector('.switch button') as HTMLButtonElement
    btn.click()
    await el.updateComplete
    expect(el.mode).toBe('sign-up')
    expect(listener).toHaveBeenCalledTimes(1)
    expect((listener.mock.calls[0][0] as CustomEvent).detail).toEqual({ mode: 'sign-up' })
  })

  it('reacts to ship-tab-change events from the segmented tabs', async () => {
    const el = await mount('<ship-auth-dialog open></ship-auth-dialog>')
    const tabs = el.renderRoot.querySelector('ship-tabs') as HTMLElement
    tabs.dispatchEvent(new CustomEvent('ship-tab-change', {
      detail: { name: 'sign-up' }, bubbles: true, composed: true,
    }))
    await el.updateComplete
    expect(el.mode).toBe('sign-up')
  })

  it('emits ship-auth-submit with email and password on form submit', async () => {
    const el = await mount('<ship-auth-dialog open></ship-auth-dialog>')
    const listener = vi.fn()
    el.addEventListener('ship-auth-submit', listener)
    const form = el.renderRoot.querySelector('form') as HTMLFormElement
    const email = form.elements.namedItem('email') as HTMLInputElement
    const password = form.elements.namedItem('password') as HTMLInputElement
    email.value = 'jane@example.com'
    password.value = 'supersecret'
    form.dispatchEvent(new Event('submit', { cancelable: true, bubbles: true }))
    expect(listener).toHaveBeenCalledTimes(1)
    expect((listener.mock.calls[0][0] as CustomEvent).detail).toEqual({
      mode: 'sign-in',
      email: 'jane@example.com',
      password: 'supersecret',
    })
  })

  it('reflects the open attribute to the nested ship-dialog', async () => {
    const el = await mount('<ship-auth-dialog open></ship-auth-dialog>')
    const dialog = el.renderRoot.querySelector('ship-dialog') as HTMLElement
    expect(dialog.hasAttribute('open')).toBe(true)
    el.open = false
    await el.updateComplete
    expect(dialog.hasAttribute('open')).toBe(false)
  })

  it('marks autocomplete new-password in sign-up mode', async () => {
    const el = await mount('<ship-auth-dialog open mode="sign-up"></ship-auth-dialog>')
    const pw = el.renderRoot.querySelector('input[type="password"]') as HTMLInputElement
    expect(pw.getAttribute('autocomplete')).toBe('new-password')
  })
})
