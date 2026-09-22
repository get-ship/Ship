<scope>
  props: {
    active = ''
    placeholder = 'Pick a collection'
  }
</scope>
<controller>
import { controller, derived, onCleanup, query, signal, unwrapOr } from '@ship/sail-runtime'
import { ship } from '@ship/sdk-client'
import type { Scope } from './ship-collection-picker.sail'

// Compact picker over ship.collections.list(). Mirrors the @ship/spaces
// switcher shape: button-as-trigger + listbox of options. Picking a
// row emits `ship-collection-pick { id }` and the host flips the active
// pointer.
interface CollectionRow {
  id: string
  name: string
  item_count: number
  system: boolean
}

const EMPTY = { items: [] as { id: string; name: string | null; item_count: number; builtin?: boolean }[] }

export default controller((ctx: Scope, host?: Element) => {

    const list = query(() => ship.collections.list())
    const rows = derived<CollectionRow[]>(() =>
      unwrapOr(list, EMPTY).items.map((c) => ({
        id: c.id,
        name: c.name ?? c.id,
        item_count: c.item_count,
        system: Boolean(c.builtin),
      })),
    )
    const open = signal(false)
    const activeLabel = derived(() => {
      const found = rows().find((c) => c.id === ctx.active())
      return found?.name ?? ''
    })

    const close = (): void => { open.set(false) }

    const onClick = (e: Event): void => {
      if (!host) return
      for (const node of e.composedPath()) {
        if (!(node instanceof Element)) continue
        if (node.matches('[data-action="toggle"]')) {
          open.set(!open())
          return
        }
        if (node.matches('[data-id]')) {
          const id = node.getAttribute('data-id')
          if (id) {
            host.dispatchEvent(
              new CustomEvent('ship-collection-pick', {
                detail: { id },
                bubbles: true,
                composed: true,
              }),
            )
          }
          close()
          return
        }
      }
    }
    const onDocClick = (e: Event): void => {
      if (!open()) return
      if (host && !e.composedPath().includes(host)) close()
    }
    const onKey = (e: KeyboardEvent): void => {
      if (e.key === 'Escape' && open()) close()
    }

    host?.addEventListener('click', onClick)
    document.addEventListener('click', onDocClick, true)
    document.addEventListener('keydown', onKey)

    // Reflect the `active` attribute back to the host (consumers can
    // either read it directly or hook the ship-collection-pick event).
    const observer = new MutationObserver(() => {
      const a = host?.getAttribute('active') ?? ''
      const found = rows().find((c) => c.id === a)
      host?.setAttribute('data-label', found?.name ?? '')
    })
    if (host) observer.observe(host, { attributes: true, attributeFilter: ['active'] })

    onCleanup(() => {
      host?.removeEventListener('click', onClick)
      document.removeEventListener('click', onDocClick, true)
      document.removeEventListener('keydown', onKey)
      observer.disconnect()
    })

    return { list, rows, open, activeLabel }
})
</controller>

<!--
  <ship-collection-picker> — @ship/collections · compact collection
  selector. Calls ship.collections.list() via the controller and
  emits ship-collection-pick { id } on selection. Pure composition
  over a native <select>-style list (no <select> primitive in
  the framework, so we render a labelled <button> list — the same
  shape the spaces pack's switcher uses, kept minimal for compact
  pickers embedded in other surfaces).

  Tag prefix `ship-*` marks this component as depending on the
  Ship runtime: mounting without a host that has called
  setShipClient() will throw on the first SDK access.
-->
<!-- <prop name="active" type="string" default="" /> -->
<!-- <prop name="placeholder" type="string" default="Pick a collection" /> -->

<match {list()}>
  {loading =>
    <div data-test="skeleton"><spinner shape="line" size="sm"></spinner></div>
  }
  {error as err => <error-state title="Couldn't load collections" body={err?.message ?? 'Request failed'}></error-state>}
  {ok as _ => }
</match>
<if {list().kind === 'ok'}>
  <button type="button" data-test="trigger"
    data-action="toggle" aria-haspopup="listbox" aria-expanded={open() ? 'true' : 'false'}
  >
    <span data-empty={activeLabel() ? 'false' : 'true'} data-test="label">
      {activeLabel() ? activeLabel() : placeholder()}
    </span>
    <span aria-hidden="true">▾</span>
  </button>
  <if {open()}>
    <div role="listbox" data-test="menu">
      <for {rows() as c} key={c.id}>
        <button type="button" role="option" data-test="row"
          data-active={c.id === active()}
          aria-selected={c.id === active() ? 'true' : 'false'}
          data-id={c.id}
        >
          <span>{c.name}</span>
          <if {c.system}><span>system</span></if>
          <span>{c.item_count}</span>
        </button>
      </for>
    </div>
  </if>
</if>
