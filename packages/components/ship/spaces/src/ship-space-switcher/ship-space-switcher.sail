<controller>
import { controller, signal } from '@ship/sail-runtime'
import type { Scope } from './ship-space-switcher.sail'

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
export default controller({
  scope: (_ctx: Scope) => ({
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
          host.dispatchEvent(new CustomEvent('ship-add', { bubbles: true, composed: true }))
          close()
          return
        }
        if (node.matches('.row[data-id]')) {
          const id = node.getAttribute('data-id')
          if (id && id !== host.getAttribute('active')) {
            host.dispatchEvent(
              new CustomEvent('ship-activate', { detail: { id }, bubbles: true, composed: true }),
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
</controller>

<!--
  ship-space-switcher.sail — the @ship/spaces SWITCHER (SHELL side).
  An always-present picker over SpaceService.List: a trigger showing the
  active space, opening a menu of registered spaces (active marked). Choosing
  one emits `ship-activate` {id} (the consumer re-points the daemon-global
  default pointer via Activate / surfaces the space's window); the register row
  emits `ship-add`. Like the registry, the surface shows no RPC/stream/daemon
  machinery.

  Rows + the active label are fed by the controller from the `spaces` (JSON)
  and `active` attributes. `open` is reflected so the host can drive it.
  Composes <icon> for the chevron / check / register glyphs. Compiled as
  <ship-space-switcher>. Parts: trigger · menu · row · add.
-->
<!-- <prop name="active" type="string" default="" /> -->
<!-- <prop name="open" type="bool" default="false" reflect /> -->
<!-- <prop name="placeholder" type="string" default="Select space" /> -->

<!-- style block removed: use sx / external CSS -->

<button type="button" data-action="toggle" aria-haspopup="listbox" aria-expanded={open() ? 'true' : 'false'}>
  <span data-cold={activeLabel() ? 'false' : 'true'}></span>
  <span data-empty={activeLabel() ? 'false' : 'true'}>{activeLabel() ? activeLabel() : placeholder()}</span>
  <icon name="chevron-down" size="sm"></icon>
</button>

<if cond={open()}>
  <div role="listbox">
    <for as="s" of={rows()} key={s.id}>
      <button type="button" role="option" data-id={s.id} data-active={s.id === active()} aria-selected={s.id === active() ? 'true' : 'false'}>
        <span data-active={s.id === active() ? 'true' : 'false'}></span>
        <span>
          <span>{s.name}</span>
          <span>{s.path}</span>
        </span>
        <icon name="check" size="sm"></icon>
      </button>
    </for>
    <div></div>
    <button type="button" data-action="add">
      <icon name="plus" size="sm"></icon>
      Register a space…
    </button>
  </div>
</if>
