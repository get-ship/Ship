import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './ship-space-switcher.sui'

interface SpaceRow {
  id: string
  name: string
  path: string
}

interface Space {
  id: string
  path: string
  name: string
}

// Property-access runtime: read `state.x`, write `state.x = …`.
export default controller<Scope>({
  scope: () => ({
    rows: signal<SpaceRow[]>([]),
    activeLabel: signal<string>(''),
  }),

  mount(state, host) {
    const sync = () => {
      let spaces: Space[] = []
      try {
        spaces = JSON.parse(host.getAttribute('spaces') || '[]') as Space[]
      } catch {
        spaces = []
      }
      state.rows = spaces.map((s) => ({ id: s.id, name: s.name, path: s.path }))
      const activeId = host.getAttribute('active') || ''
      const found = spaces.find((s) => s.id === activeId)
      state.activeLabel = found ? found.name : ''
    }
    sync()
    const observer = new MutationObserver(sync)
    observer.observe(host, { attributes: true, attributeFilter: ['spaces', 'active'] })

    const close = () => {
      state.open = false
    }

    const onClick = (e: Event) => {
      for (const node of e.composedPath()) {
        if (!(node instanceof Element)) continue
        if (node.matches('[data-action="toggle"]')) {
          state.open = !host.hasAttribute('open')
          return
        }
        if (node.matches('[data-action="add"]')) {
          host.dispatchEvent(new CustomEvent('sui-add', { bubbles: true, composed: true }))
          close()
          return
        }
        if (node.matches('.row[data-id]')) {
          const id = node.getAttribute('data-id')
          if (id && id !== host.getAttribute('active')) {
            host.dispatchEvent(
              new CustomEvent('sui-activate', { detail: { id }, bubbles: true, composed: true }),
            )
          }
          close()
          return
        }
      }
    }
    host.addEventListener('click', onClick)

    // Dismiss on outside click / Escape.
    const onDocClick = (e: Event) => {
      if (!host.hasAttribute('open')) return
      if (!e.composedPath().includes(host)) close()
    }
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && host.hasAttribute('open')) close()
    }
    document.addEventListener('click', onDocClick, true)
    document.addEventListener('keydown', onKey)

    return () => {
      observer.disconnect()
      host.removeEventListener('click', onClick)
      document.removeEventListener('click', onDocClick, true)
      document.removeEventListener('keydown', onKey)
    }
  },
})
