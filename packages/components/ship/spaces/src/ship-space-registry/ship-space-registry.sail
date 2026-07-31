<controller>
import { controller, signal } from '@ship/sail-runtime'
import type { Scope } from './ship-space-registry.sail'

/** A registry row's view model — the `{#for}` row. Derived from a
 *  SpaceService `Space` message; `lastSeen` is the relative form of
 *  `last_seen_at` computed at parse time. */
interface SpaceRow {
  id: string
  name: string
  path: string
  lastSeen: string
}

/** SpaceService.Space (List / Watch(registry) payload shape). */
interface Space {
  id: string
  path: string
  name: string
  created_at?: string
  last_seen_at?: string
}

/** Relative last-seen, e.g. "now", "3h", "7d". Anchored to the caller's
 *  clock; the daemon returns RFC-3339 timestamps. */
function relTime(iso: string | undefined, now: number = Date.now()): string {
  if (!iso) return ''
  const then = Date.parse(iso)
  if (Number.isNaN(then)) return ''
  const mins = Math.max(0, Math.round((now - then) / 60000))
  if (mins < 5) return 'now'
  if (mins < 60) return mins + 'm'
  const hrs = Math.round(mins / 60)
  if (hrs < 24) return hrs + 'h'
  const days = Math.round(hrs / 24)
  if (days < 7) return days + 'd'
  return Math.round(days / 7) + 'w'
}

// State API note: this targets the property-access runtime — read `state.rows`,
// write `state.rows = …` (the mount-state proxy routes the write to the signal).
export default controller({
  scope: (_ctx: Scope) => ({ rows: signal<SpaceRow[]>([]) }),

  mount(state, host) {
    // Parse the `spaces` attribute (a SpaceList.spaces[] JSON array) into rows.
    // The integration layer keeps this attribute in sync with List + the
    // Watch(registry) snapshot/patches; the surface itself never sees a stream.
    const load = () => {
      let spaces: Space[] = []
      try {
        spaces = JSON.parse(host.getAttribute('spaces') || '[]') as Space[]
      } catch {
        spaces = []
      }
      state.rows = spaces.map((s) => ({
        id: s.id,
        name: s.name,
        path: s.path,
        lastSeen: relTime(s.last_seen_at),
      }))
    }
    load()
    const observer = new MutationObserver(load)
    observer.observe(host, { attributes: true, attributeFilter: ['spaces'] })

    // Delegated clicks: composedPath() reaches the real shadow target even
    // though the event retargets to the host outside the shadow root.
    const onClick = (e: Event) => {
      for (const node of e.composedPath()) {
        if (!(node instanceof Element)) continue
        if (node.matches('[data-action="add"]')) {
          host.dispatchEvent(new CustomEvent('ship-add', { bubbles: true, composed: true }))
          return
        }
        if (node.matches('.row[data-id]')) {
          const id = node.getAttribute('data-id')
          if (id && id !== host.getAttribute('active')) {
            host.dispatchEvent(
              new CustomEvent('ship-activate', { detail: { id }, bubbles: true, composed: true }),
            )
          }
          return
        }
      }
    }
    host.addEventListener('click', onClick)

    return () => {
      observer.disconnect()
      host.removeEventListener('click', onClick)
    }
  },
})
</controller>

<!--
  ship-space-registry.sail — the @ship/spaces REGISTRY surface (SHELL side).
  Renders SpaceService.List (spaces[] + active_id) as a selectable list:
  each row is a registered space (name · path · last-seen), the active one
  marked. Row click → `ship-activate` {id} (consumer re-points the daemon-global
  default pointer via Activate). The register affordance → `ship-add` (consumer
  opens the add flow).

  Presentational + token-pure. The rows + relative last-seen are fed by the
  controller from the `spaces` attribute (JSON) and `active` (the active id);
  no daemon, RPC, or Watch machinery is visible in the surface itself — those
  live in the integration layer that owns this element. Compiled as
  <ship-space-registry>. Parts: container · header · add · rows · row · empty.
-->
<!-- <prop name="active" type="string" default="" /> -->
<!-- <prop name="heading" type="string" default="Spaces" /> -->

<!-- style block removed: use sx / external CSS -->

<div>
  <header>
    <span>{heading()} <span>{rows().length}</span></span>
    <span></span>
    <if cond={rows().length > 0}>
      <button type="button" data-action="add">Register a space</button>
    </if>
  </header>

  <if cond={rows().length === 0}>
    <div>
      <span>No spaces registered</span>
      <span>Register a directory to get started. Each space is an independent Ship context.</span>
      <button type="button" data-action="add">Register a space</button>
    </div>
  <else>
    <div role="list">
      <for as="s" of={rows()} key={s.id}>
        <button
          type="button"
          role="listitem"
          data-id={s.id}
          data-active={s.id === active()}
          aria-current={s.id === active() ? 'true' : 'false'}>
          <span data-active={s.id === active() ? 'true' : 'false'}></span>
          <span>{s.name}</span>
          <span>{s.path}</span>
          <span>{s.lastSeen}</span>
          <span>{s.id === active() ? 'active' : 'open'}</span>
        </button>
      </for>
    </div>
  </else></if>
</div>
