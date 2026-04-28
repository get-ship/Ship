import { describe, expect, it, beforeAll, afterEach } from 'vitest'
import { loadSailComponent } from './helpers'

describe('ship-drawer (sail)', () => {
  let tag: string

  beforeAll(async () => {
    tag = await loadSailComponent('ship-drawer.set')
  }, 120_000)

  afterEach(() => {
    // Clean up any drawer left in the DOM and reset body inline styles
    // touched by the scroll-lock helper (test isolation).
    document.querySelectorAll(tag).forEach((n) => n.remove())
    delete document.body.dataset.shipDrawerLock
    delete document.body.dataset.shipDrawerOverflow
    delete document.body.dataset.shipDrawerPad
    document.body.style.overflow = ''
    document.body.style.paddingRight = ''
  })

  function mount(markup: string): HTMLElement {
    const wrap = document.createElement('div')
    wrap.innerHTML = markup
    const el = wrap.firstElementChild as HTMLElement
    document.body.appendChild(el)
    return el
  }

  it('renders a <dialog> with header / body / footer regions', () => {
    const el = mount(`<${tag}></${tag}>`)
    const dialog = el.querySelector('dialog')
    expect(dialog).not.toBeNull()
    expect(dialog!.getAttribute('role')).toBe('dialog')
    expect(dialog!.getAttribute('aria-modal')).toBe('true')
    expect(el.querySelector('.header')).not.toBeNull()
    expect(el.querySelector('.body')).not.toBeNull()
    expect(el.querySelector('.footer')).not.toBeNull()
    expect(el.querySelector('.close')).not.toBeNull()
  })

  it('starts closed: data-state=closed and no [open] reflected', () => {
    const el = mount(`<${tag}></${tag}>`)
    expect(el.getAttribute('data-state')).toBe('closed')
    expect(el.hasAttribute('open')).toBe(false)
  })

  it('show() / close() toggle data-state and [open]', () => {
    const el = mount(`<${tag}></${tag}>`) as HTMLElement & {
      show: () => void
      close: () => void
    }
    el.show()
    expect(el.getAttribute('data-state')).toBe('open')
    expect(el.hasAttribute('open')).toBe(true)
    el.close()
    expect(el.getAttribute('data-state')).toBe('closed')
    expect(el.hasAttribute('open')).toBe(false)
  })

  it('initial [open] attribute opens the drawer and dispatches ship-open', () => {
    let opened = 0
    document.addEventListener('ship-open', () => opened++, { once: true })
    const el = mount(`<${tag} open></${tag}>`)
    expect(el.getAttribute('data-state')).toBe('open')
    expect(opened).toBe(1)
  })

  it('default side is right; side="left" reflects on the host', () => {
    const a = mount(`<${tag}></${tag}>`)
    expect(a.getAttribute('side')).toBeNull() // default applied via signal, not reflected
    const b = mount(`<${tag} side="left"></${tag}>`)
    expect(b.getAttribute('side')).toBe('left')
  })

  it('size attribute reflects to host', () => {
    for (const size of ['sm', 'md', 'lg']) {
      const el = mount(`<${tag} size="${size}"></${tag}>`)
      expect(el.getAttribute('size')).toBe(size)
      el.remove()
    }
  })

  it('close button emits ship-dismiss + ship-close and closes', () => {
    const el = mount(`<${tag} open></${tag}>`)
    const events: string[] = []
    el.addEventListener('ship-dismiss', () => events.push('dismiss'))
    el.addEventListener('ship-close', () => events.push('close'))
    const btn = el.querySelector('.close') as HTMLButtonElement
    btn.click()
    expect(el.getAttribute('data-state')).toBe('closed')
    expect(events).toContain('dismiss')
    expect(events).toContain('close')
  })

  it('close button dismisses even when dismissible="false"', () => {
    const el = mount(`<${tag} open dismissible="false"></${tag}>`)
    const btn = el.querySelector('.close') as HTMLButtonElement
    btn.click()
    expect(el.getAttribute('data-state')).toBe('closed')
  })

  it('backdrop click closes when dismissible (default)', () => {
    const el = mount(`<${tag} open></${tag}>`)
    const dialog = el.querySelector('dialog') as HTMLDialogElement
    const evt = new MouseEvent('click', { bubbles: true, composed: true })
    Object.defineProperty(evt, 'target', { value: dialog, writable: false })
    dialog.dispatchEvent(evt)
    expect(el.getAttribute('data-state')).toBe('closed')
  })

  it('backdrop click is a no-op when dismissible="false"', () => {
    const el = mount(`<${tag} open dismissible="false"></${tag}>`)
    const dialog = el.querySelector('dialog') as HTMLDialogElement
    const evt = new MouseEvent('click', { bubbles: true, composed: true })
    Object.defineProperty(evt, 'target', { value: dialog, writable: false })
    dialog.dispatchEvent(evt)
    expect(el.getAttribute('data-state')).toBe('open')
  })

  it('clicks on inner content do not close the drawer', () => {
    const el = mount(`<${tag} open><span class="payload">hi</span></${tag}>`)
    const dialog = el.querySelector('dialog') as HTMLDialogElement
    const inner = el.querySelector('.payload') as HTMLElement
    const evt = new MouseEvent('click', { bubbles: true, composed: true })
    Object.defineProperty(evt, 'target', { value: inner, writable: false })
    dialog.dispatchEvent(evt)
    expect(el.getAttribute('data-state')).toBe('open')
  })

  it('Escape on the dialog (cancel→close) dismisses by default', () => {
    const el = mount(`<${tag} open></${tag}>`)
    const dialog = el.querySelector('dialog') as HTMLDialogElement
    // Native <dialog> raises `cancel` on Escape, default action closes
    // and fires `close`. We simulate the close event directly because
    // jsdom doesn't synthesize the cancel→close pipeline.
    let closed = 0
    el.addEventListener('ship-close', () => closed++)
    dialog.dispatchEvent(new Event('close'))
    expect(closed).toBe(1)
    expect(el.getAttribute('data-state')).toBe('closed')
  })

  it('Escape is suppressed when dismissible="false" (cancel preventDefault)', () => {
    const el = mount(`<${tag} open dismissible="false"></${tag}>`)
    const dialog = el.querySelector('dialog') as HTMLDialogElement
    const cancel = new Event('cancel', { cancelable: true })
    dialog.dispatchEvent(cancel)
    expect(cancel.defaultPrevented).toBe(true)
    // Drawer is still open because we cancelled the close path.
    expect(el.getAttribute('data-state')).toBe('open')
  })

  it('routes title / description / default / footer slots into their wells', () => {
    const el = mount(`
      <${tag}>
        <span slot="title">Edit item</span>
        <span slot="description">Editing row abc</span>
        <p class="payload">Body content</p>
        <button slot="footer">Save</button>
      </${tag}>
    `)
    expect(el.querySelector('.title')!.textContent).toContain('Edit item')
    expect(el.querySelector('.description')!.textContent).toContain('Editing row abc')
    expect(el.querySelector('.body')!.textContent).toContain('Body content')
    expect(el.querySelector('.footer')!.textContent).toContain('Save')
  })

  it('header carries aria-labelledby/described-by ids that point at the rendered ids', () => {
    const el = mount(`<${tag}></${tag}>`)
    const dialog = el.querySelector('dialog') as HTMLDialogElement
    const header = el.querySelector('.header') as HTMLElement
    const body = el.querySelector('.body') as HTMLElement
    expect(header.id).toMatch(/^ship-drawer-header-/)
    expect(body.id).toMatch(/^ship-drawer-body-/)
    expect(dialog.getAttribute('aria-labelledby')).toBe(header.id)
    expect(dialog.getAttribute('aria-describedby')).toBe(body.id)
  })

  it('reopening after close re-applies showModal and re-fires ship-open', () => {
    const el = mount(`<${tag}></${tag}>`) as HTMLElement & { show(): void; close(): void }
    let opens = 0
    el.addEventListener('ship-open', () => opens++)
    el.show()
    el.close()
    el.show()
    expect(opens).toBe(2)
    expect(el.getAttribute('data-state')).toBe('open')
  })

  it('multiple drawers stack the body scroll-lock counter', () => {
    const a = mount(`<${tag} open></${tag}>`)
    const b = mount(`<${tag} open></${tag}>`)
    expect(document.body.dataset.shipDrawerLock).toBe('2')
    ;(a as any).close()
    expect(document.body.dataset.shipDrawerLock).toBe('1')
    ;(b as any).close()
    expect(document.body.dataset.shipDrawerLock).toBeUndefined()
  })

  it('dismissible defaults to true (drawer is dismissible without the attribute)', () => {
    const el = mount(`<${tag} open></${tag}>`)
    const dialog = el.querySelector('dialog') as HTMLDialogElement
    const evt = new MouseEvent('click', { bubbles: true, composed: true })
    Object.defineProperty(evt, 'target', { value: dialog, writable: false })
    dialog.dispatchEvent(evt)
    expect(el.getAttribute('data-state')).toBe('closed')
  })

  it('size × side matrix renders without error', () => {
    const sizes = ['sm', 'md', 'lg']
    const sides = ['left', 'right']
    let n = 0
    for (const size of sizes) {
      for (const side of sides) {
        const el = mount(`<${tag} size="${size}" side="${side}"></${tag}>`)
        expect(el.querySelector('dialog')).not.toBeNull()
        el.remove()
        n++
      }
    }
    expect(n).toBe(sizes.length * sides.length)
  })
})
