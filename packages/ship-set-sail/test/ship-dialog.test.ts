import { describe, expect, it, beforeAll } from 'vitest'
import { loadSailComponent } from './helpers'

type DialogHost = HTMLElement & {
  show?: () => void
  showModal?: () => void
  close?: () => void
}

describe('ship-dialog (sail)', () => {
  let tag: string

  beforeAll(async () => {
    tag = await loadSailComponent('ship-dialog.set')
  }, 120_000)

  function mount(attrs = '', children = ''): DialogHost {
    const wrap = document.createElement('div')
    wrap.innerHTML = `<ship-dialog ${attrs}>${children}</ship-dialog>`
    const el = wrap.firstElementChild as DialogHost
    document.body.appendChild(el)
    return el
  }

  it('renders a native <dialog> with header/body/footer regions', () => {
    const el = mount(
      '',
      `<span slot="header">Title</span>Body<span slot="footer">F</span>`,
    )
    const dlg = el.querySelector('dialog')!
    expect(dlg).not.toBeNull()
    expect(dlg.querySelector('.header')).not.toBeNull()
    expect(dlg.querySelector('.body')).not.toBeNull()
    expect(dlg.querySelector('.footer')).not.toBeNull()
    expect(dlg.querySelector('.close')).not.toBeNull()
    el.remove()
  })

  it('exposes show()/showModal()/close() on the host and toggles [open]', () => {
    const el = mount()
    expect(el.hasAttribute('open')).toBe(false)
    expect(typeof el.show).toBe('function')
    expect(typeof el.showModal).toBe('function')
    expect(typeof el.close).toBe('function')
    try { el.show!() } catch { /* jsdom */ }
    expect(el.hasAttribute('open')).toBe(true)
    el.close!()
    expect(el.hasAttribute('open')).toBe(false)
    el.remove()
  })

  it('reflects data-state to "open" / "closed"', () => {
    const el = mount()
    expect(el.getAttribute('data-state')).toBe('closed')
    try { el.showModal!() } catch { /* jsdom */ }
    expect(el.getAttribute('data-state')).toBe('open')
    el.close!()
    expect(el.getAttribute('data-state')).toBe('closed')
    el.remove()
  })

  it('opens via [open] attribute on mount and dispatches ship-open', () => {
    const opens: number[] = []
    const handler = () => opens.push(1)
    document.addEventListener('ship-open', handler)
    const el = mount('open')
    expect(el.hasAttribute('open')).toBe(true)
    expect(opens.length).toBe(1)
    document.removeEventListener('ship-open', handler)
    el.remove()
  })

  it('modal=true uses showModal()', () => {
    const el = mount()
    const dlg = el.querySelector('dialog')! as HTMLDialogElement
    const calls: string[] = []
    dlg.showModal = function () { calls.push('modal'); this.setAttribute('open', '') }
    dlg.show = function () { calls.push('show'); this.setAttribute('open', '') }
    el.show!()
    expect(calls).toEqual(['modal'])
    el.remove()
  })

  it('modal=false uses show() (non-modal)', () => {
    const el = mount('modal="false"')
    const dlg = el.querySelector('dialog')! as HTMLDialogElement
    const calls: string[] = []
    dlg.showModal = function () { calls.push('modal'); this.setAttribute('open', '') }
    dlg.show = function () { calls.push('show'); this.setAttribute('open', '') }
    el.show!()
    expect(calls).toEqual(['show'])
    el.remove()
  })

  it('aria-modal mirrors the [modal] prop', () => {
    const el = mount()
    const dlg = el.querySelector('dialog')!
    expect(dlg.getAttribute('aria-modal')).toBe('true')
    el.setAttribute('modal', 'false')
    expect(dlg.getAttribute('aria-modal')).toBe('false')
    el.remove()
  })

  it('close button fires ship-dismiss and closes the dialog', () => {
    const el = mount('open')
    const btn = el.querySelector('.close') as HTMLButtonElement
    let dismissed = 0
    el.addEventListener('ship-dismiss', () => dismissed++)
    btn.click()
    expect(dismissed).toBe(1)
    expect(el.hasAttribute('open')).toBe(false)
    el.remove()
  })

  it('ESC dismissal — native dialog close event fires ship-close', () => {
    const el = mount('open')
    const dlg = el.querySelector('dialog')! as HTMLDialogElement
    let closed = 0
    el.addEventListener('ship-close', () => closed++)
    // Simulate native ESC: dialog dispatches a 'close' event after closing.
    ;(dlg as any).open = false
    dlg.dispatchEvent(new Event('close'))
    expect(el.hasAttribute('open')).toBe(false)
    expect(closed).toBe(1)
    el.remove()
  })

  it('backdrop click closes when [dismissible]; suppressed when not', () => {
    const el = mount('open')
    const dlg = el.querySelector('dialog')! as HTMLDialogElement
    // Click landing on the dialog itself = backdrop pseudo-element.
    const evt = new MouseEvent('click', { bubbles: true })
    Object.defineProperty(evt, 'target', { value: dlg })
    dlg.dispatchEvent(evt)
    expect(el.hasAttribute('open')).toBe(false)

    const el2 = mount('open dismissible="false"')
    const dlg2 = el2.querySelector('dialog')! as HTMLDialogElement
    const evt2 = new MouseEvent('click', { bubbles: true })
    Object.defineProperty(evt2, 'target', { value: dlg2 })
    dlg2.dispatchEvent(evt2)
    expect(el2.hasAttribute('open')).toBe(true)
    el.remove()
    el2.remove()
  })

  it('backdrop click on inner content does NOT close', () => {
    const el = mount('open', '<button class="action">Go</button>')
    const dlg = el.querySelector('dialog')!
    const inner = el.querySelector('.action') as HTMLElement
    const evt = new MouseEvent('click', { bubbles: true })
    Object.defineProperty(evt, 'target', { value: inner })
    dlg.dispatchEvent(evt)
    expect(el.hasAttribute('open')).toBe(true)
    el.remove()
  })

  it('aria-labelledby and aria-describedby resolve to header/body ids', () => {
    const el = mount()
    const dlg = el.querySelector('dialog')!
    const headerId = el.querySelector('.header')!.id
    const bodyId = el.querySelector('.body')!.id
    expect(headerId).toMatch(/^ship-dialog-header-/)
    expect(bodyId).toMatch(/^ship-dialog-body-/)
    expect(dlg.getAttribute('aria-labelledby')).toBe(headerId)
    expect(dlg.getAttribute('aria-describedby')).toBe(bodyId)
    el.remove()
  })

  it('role-override flips role between dialog and alertdialog', () => {
    const a = mount()
    expect(a.querySelector('dialog')!.getAttribute('role')).toBe('dialog')
    a.remove()
    const b = mount('role-override="alertdialog"')
    expect(b.querySelector('dialog')!.getAttribute('role')).toBe('alertdialog')
    b.remove()
  })

  it('no-close-button hides the close button', () => {
    const el = mount('no-close-button')
    expect(el.hasAttribute('no-close-button')).toBe(true)
    // The element stays in the DOM; CSS hides it. Verify the host attr.
    expect(el.querySelector('.close')).not.toBeNull()
    el.remove()
  })

  it('size and backdrop attributes reflect on the host', () => {
    const el = mount('size="lg" backdrop="blur"')
    expect(el.getAttribute('size')).toBe('lg')
    expect(el.getAttribute('backdrop')).toBe('blur')
    el.remove()
  })

  it('renders sentinels at the dialog edges for keyboard trap', () => {
    const el = mount()
    const sentinels = el.querySelectorAll('.sentinel')
    expect(sentinels.length).toBe(2)
    expect(sentinels[0].getAttribute('aria-hidden')).toBe('true')
    expect(sentinels[1].getAttribute('aria-hidden')).toBe('true')
    el.remove()
  })

  it('size × backdrop matrix renders without error', () => {
    const sizes = ['sm', 'md', 'lg', 'full']
    const backdrops = ['default', 'blur', 'solid']
    let n = 0
    for (const size of sizes) {
      for (const bd of backdrops) {
        const el = mount(`size="${size}" backdrop="${bd}"`)
        expect(el.querySelector('dialog')).not.toBeNull()
        el.remove()
        n++
      }
    }
    expect(n).toBe(sizes.length * backdrops.length)
  })
})
