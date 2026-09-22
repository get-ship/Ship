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
  <ship-space-switcher>.
-->
<scope>
  props: {
    active = ''
    open = false
    placeholder = 'Select space'
  }
</scope>

<controller>
import { controller, signal, onCleanup } from '@ship/sail-runtime'
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

export default controller((ctx: Scope, host?: Element) => {
  const rows = signal<SpaceRow[]>([])
  const activeLabel = signal<string>('')
  if (!host) return { rows, activeLabel }

  const sync = () => {
    let spaces: Space[] = []
    try {
      spaces = JSON.parse(host.getAttribute('spaces') || '[]') as Space[]
    } catch {
      spaces = []
    }
    rows.set(spaces.map((s) => ({ id: s.id, name: s.name, path: s.path })))
    const activeId = host.getAttribute('active') || ''
    const found = spaces.find((s) => s.id === activeId)
    activeLabel.set(found ? found.name : '')
  }
  sync()
  const observer = new MutationObserver(sync)
  observer.observe(host, { attributes: true, attributeFilter: ['spaces', 'active'] })

  const close = () => {
    ctx.open.set(false)
  }

  const onClick = (e: Event) => {
    for (const node of e.composedPath()) {
      if (!(node instanceof Element)) continue
      if (node.matches('[data-action="toggle"]')) {
        ctx.open.set(!host.hasAttribute('open'))
        return
      }
      if (node.matches('[data-action="add"]')) {
        host.dispatchEvent(new CustomEvent('ship-add', { bubbles: true, composed: true }))
        close()
        return
      }
      if (node.matches('[data-id]')) {
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

  const onDocClick = (e: Event) => {
    if (!host.hasAttribute('open')) return
    if (!e.composedPath().includes(host)) close()
  }
  const onKey = (e: KeyboardEvent) => {
    if (e.key === 'Escape' && host.hasAttribute('open')) close()
  }
  document.addEventListener('click', onDocClick, true)
  document.addEventListener('keydown', onKey)

  onCleanup(() => {
    observer.disconnect()
    host.removeEventListener('click', onClick)
    document.removeEventListener('click', onDocClick, true)
    document.removeEventListener('keydown', onKey)
  })

  return { rows, activeLabel }
})
</controller>

<button type="button" data-action="toggle" aria-haspopup="listbox" aria-expanded={open() ? 'true' : 'false'}>
  <span data-cold={activeLabel() ? 'false' : 'true'}></span>
  <span data-empty={activeLabel() ? 'false' : 'true'}>{activeLabel() ? activeLabel() : placeholder()}</span>
  <icon name="chevron-down" size="sm"></icon>
</button>

<if {open()}>
  <div role="listbox">
    <for {rows() as s} key={s.id}>
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
