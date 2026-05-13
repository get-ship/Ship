import { afterEach, describe, expect, it, vi } from 'vitest'
import '../src/sui-auth-dialog/sui-auth-dialog.compiled.js'
import '../src/sui-dialog/sui-dialog.compiled.js'
import '../src/sui-tabs/sui-tabs.compiled.js'
import '../src/sui-tabs/sui-tab.compiled.js'
import '../src/sui-button/sui-button.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-auth-dialog>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-auth-dialog')).toBeDefined()
  })

  it('renders a sui-dialog with sui-tabs inside', async () => {
    const { el } = mount(reg, 'sui-auth-dialog')
    await flush()
    const dlg = shadow(el).querySelector('sui-dialog')
    expect(dlg).not.toBeNull()
    expect(shadow(el).querySelector('sui-tabs')).not.toBeNull()
    expect(shadow(el).querySelectorAll('sui-tab').length).toBe(2)
  })

  it('defaults mode to sign-in and shows the welcome title', async () => {
    const { el } = mount(reg, 'sui-auth-dialog')
    await flush()
    // sui-tabs reflects `active` from the bound mode signal; the inner
    // dialog renders the derived defaultTitle.
    expect(shadow(el).querySelector('sui-tabs')?.getAttribute('active')).toBe('sign-in')
    const dlg = shadow(el).querySelector('sui-dialog')!
    expect(dlg.querySelector('[slot="header"]')?.textContent).toContain('Welcome back')
  })

  it('switches title when mode flips to sign-up', async () => {
    const { el } = mount(reg, 'sui-auth-dialog', { mode: 'sign-up' })
    await flush()
    expect(shadow(el).querySelector('sui-tabs')?.getAttribute('active')).toBe('sign-up')
    const dlg = shadow(el).querySelector('sui-dialog')!
    expect(dlg.querySelector('[slot="header"]')?.textContent).toContain('Create your account')
  })

  it('emits sui-mode-change when the user clicks the switch link', async () => {
    const { el } = mount(reg, 'sui-auth-dialog', { mode: 'sign-in' })
    await flush()
    const handler = vi.fn()
    el.addEventListener('sui-mode-change', handler)
    const swBtn = shadow(el).querySelector('.switch button') as HTMLButtonElement
    swBtn.click()
    await flush()
    expect(handler).toHaveBeenCalledTimes(1)
    expect((handler.mock.calls[0][0] as CustomEvent).detail).toEqual({ mode: 'sign-up' })
    expect(el.getAttribute('mode')).toBe('sign-up')
  })

  it('emits sui-mode-change in response to a sui-tab-change event', async () => {
    const { el } = mount(reg, 'sui-auth-dialog')
    await flush()
    const handler = vi.fn()
    el.addEventListener('sui-mode-change', handler)
    const tabs = shadow(el).querySelector('sui-tabs')!
    tabs.dispatchEvent(
      new CustomEvent('sui-tab-change', {
        detail: { name: 'sign-up' },
        bubbles: true,
        composed: true,
      }),
    )
    await flush()
    expect(handler).toHaveBeenCalledTimes(1)
    expect(el.getAttribute('mode')).toBe('sign-up')
  })

  it('emits sui-auth-submit on form submit with email + password', async () => {
    const { el } = mount(reg, 'sui-auth-dialog', { mode: 'sign-in' })
    await flush()
    const handler = vi.fn()
    el.addEventListener('sui-auth-submit', handler)
    const form = shadow(el).querySelector('form')!
    const email = form.elements.namedItem('email') as HTMLInputElement
    const pw = form.elements.namedItem('password') as HTMLInputElement
    email.value = 'ada@example.com'
    pw.value = 's3cretpw!'
    form.requestSubmit()
    expect(handler).toHaveBeenCalledTimes(1)
    expect((handler.mock.calls[0][0] as CustomEvent).detail).toEqual({
      mode: 'sign-in',
      email: 'ada@example.com',
      password: 's3cretpw!',
    })
  })

  it('uses new-password autocomplete when signing up', async () => {
    const { el } = mount(reg, 'sui-auth-dialog', { mode: 'sign-up' })
    await flush()
    const pw = shadow(el).querySelector('input[name="password"]') as HTMLInputElement
    expect(pw.getAttribute('autocomplete')).toBe('new-password')
  })

  it('uses current-password autocomplete when signing in', async () => {
    const { el } = mount(reg, 'sui-auth-dialog', { mode: 'sign-in' })
    await flush()
    const pw = shadow(el).querySelector('input[name="password"]') as HTMLInputElement
    expect(pw.getAttribute('autocomplete')).toBe('current-password')
  })

  it('shows "Signing in…" on the submit button when loading', async () => {
    const { el } = mount(reg, 'sui-auth-dialog', { mode: 'sign-in', loading: true })
    await flush()
    const btn = shadow(el).querySelector('sui-button[part="submit"]')!
    expect(btn.textContent?.trim()).toBe('Signing in…')
    expect(btn.hasAttribute('disabled')).toBe(true)
  })

  it('hides the "or" divider when no providers slotted', async () => {
    const { el } = mount(reg, 'sui-auth-dialog')
    await flush()
    expect(shadow(el).querySelector('.or')).toBeNull()
  })

  it('shows the "or" divider when providers slot has content', async () => {
    const { el } = mount(
      reg,
      'sui-auth-dialog',
      {},
      '<button slot="providers">Continue with GitHub</button>',
    )
    await flush()
    expect(shadow(el).querySelector('.or')?.textContent?.trim()).toBe('or')
  })

  it('propagates sui-close from the inner dialog and clears open', async () => {
    const { el } = mount(reg, 'sui-auth-dialog', { open: true })
    await flush()
    const closes = vi.fn()
    el.addEventListener('sui-close', closes)
    const dlg = shadow(el).querySelector('sui-dialog')!
    dlg.dispatchEvent(
      new CustomEvent('sui-close', { bubbles: true, composed: true }),
    )
    await flush()
    // The composed:true event from the inner dialog reaches the host
    // naturally; the controller does not re-dispatch. (sui-dialog's
    // own close() also emits sui-close once we drop `open`, but that
    // second emission is the dialog's own contract, not a host echo.)
    expect(closes.mock.calls.length).toBeGreaterThanOrEqual(1)
    expect(el.hasAttribute('open')).toBe(false)
  })

  it('exposes show()/showModal()/close() methods on the host', async () => {
    const { el } = mount(reg, 'sui-auth-dialog') as {
      el: HTMLElement & { show(): void; showModal(): void; close(): void }
    }
    await flush()
    expect(typeof el.show).toBe('function')
    expect(typeof el.showModal).toBe('function')
    expect(typeof el.close).toBe('function')
    el.show()
    await flush()
    expect(el.hasAttribute('open')).toBe(true)
    el.close()
    await flush()
    expect(el.hasAttribute('open')).toBe(false)
  })
})
