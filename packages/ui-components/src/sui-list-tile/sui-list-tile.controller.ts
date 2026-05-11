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
    const scope = state as unknown as Scope

    installDividerStyles(host.ownerDocument)

    const resolveRole = () => {
      if (scope.navigable()) return 'link'
      if (scope.interactive()) return 'button'
      return 'listitem'
    }

    const stopRole = effect(() => {
      scope.navigable()
      scope.interactive()
      host.setAttribute('role', resolveRole())
    })

    const stopSelected = effect(() => {
      host.setAttribute('aria-selected', scope.selected() ? 'true' : 'false')
    })

    const stopDisabled = effect(() => {
      host.setAttribute('aria-disabled', scope.disabled() ? 'true' : 'false')
    })

    const stopTabindex = effect(() => {
      const el = scope.tileEl() as HTMLElement | null
      if (!el) return
      const interactive = scope.interactive()
      const disabled = scope.disabled()
      if (interactive && !disabled) {
        el.setAttribute('tabindex', '0')
      } else {
        el.removeAttribute('tabindex')
      }
    })

    const dispatchSelect = (): boolean => {
      if (scope.disabled() || !scope.interactive()) return false
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
      if (!scope.interactive()) return
      const prevented = dispatchSelect()
      if (prevented && scope.navigable()) {
        e.preventDefault()
      }
    }

    const onKeyDown = (e: KeyboardEvent) => {
      if (!scope.interactive() || scope.disabled()) return
      if (scope.navigable() && e.key === 'Enter') return
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault()
        dispatchSelect()
      }
    }

    const stopHandlers = effect(() => {
      const el = scope.tileEl() as HTMLElement | null
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
