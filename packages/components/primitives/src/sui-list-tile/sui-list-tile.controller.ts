import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-list-tile.sui'

const DIVIDER_CSS = `
sui-list-tile[variant='row'] + sui-list-tile[variant='row'] {
  border-top: 1px solid var(--border-subtle);
}
sui-list-tile[variant='row'][selected] + sui-list-tile[variant='row'],
sui-list-tile[variant='row'] + sui-list-tile[variant='row'][selected] {
  border-top: none;
}
`

const _dividerInstalled = new WeakSet<Document>()
function installDividerStyles(doc: Document) {
  if (_dividerInstalled.has(doc)) return
  _dividerInstalled.add(doc)
  try {
    if (typeof CSSStyleSheet !== 'undefined' && 'replaceSync' in CSSStyleSheet.prototype) {
      const sheet = new CSSStyleSheet()
      sheet.replaceSync(DIVIDER_CSS)
      doc.adoptedStyleSheets = [...doc.adoptedStyleSheets, sheet]
      return
    }
  } catch {
    // fall through
  }
  const style = doc.createElement('style')
  style.setAttribute('data-sui-list-tile-dividers', '')
  style.textContent = DIVIDER_CSS
  doc.head.appendChild(style)
}

export default controller<Scope>({
  mount(state, host) {

    installDividerStyles(host.ownerDocument)

    const resolveRole = () => {
      if (state.$signals.navigable()) return 'link'
      if (state.$signals.interactive()) return 'button'
      return 'listitem'
    }

    const stopRole = effect(() => {
      state.$signals.navigable()
      state.$signals.interactive()
      host.setAttribute('role', resolveRole())
    })

    const stopSelected = effect(() => {
      host.setAttribute('aria-selected', state.$signals.selected() ? 'true' : 'false')
    })

    const stopDisabled = effect(() => {
      host.setAttribute('aria-disabled', state.$signals.disabled() ? 'true' : 'false')
    })

    const stopTabindex = effect(() => {
      const el = state.$signals.tileEl() as HTMLElement | null
      if (!el) return
      const interactive = state.$signals.interactive()
      const disabled = state.$signals.disabled()
      if (interactive && !disabled) {
        el.setAttribute('tabindex', '0')
      } else {
        el.removeAttribute('tabindex')
      }
    })

    const dispatchSelect = (): boolean => {
      if (state.$signals.disabled() || !state.$signals.interactive()) return false
      const ev = new CustomEvent('sui-select', {
        detail: {},
        bubbles: true,
        composed: true,
        cancelable: true,
      })
      host.dispatchEvent(ev)
      return ev.defaultPrevented
    }

    const onClick = (e: Event) => {
      if (!state.$signals.interactive()) return
      const prevented = dispatchSelect()
      if (prevented && state.$signals.navigable()) {
        e.preventDefault()
      }
    }

    const onKeyDown = (e: KeyboardEvent) => {
      if (!state.$signals.interactive() || state.$signals.disabled()) return
      if (state.$signals.navigable() && e.key === 'Enter') return
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault()
        dispatchSelect()
      }
    }

    const stopHandlers = effect(() => {
      const el = state.$signals.tileEl() as HTMLElement | null
      if (!el) return
      el.addEventListener('click', onClick)
      el.addEventListener('keydown', onKeyDown)
      return () => {
        el.removeEventListener('click', onClick)
        el.removeEventListener('keydown', onKeyDown)
      }
    })

    return () => {
      stopRole()
      stopSelected()
      stopDisabled()
      stopTabindex()
      stopHandlers()
    }
  },
})
