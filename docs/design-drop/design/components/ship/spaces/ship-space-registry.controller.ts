import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './ship-space-registry.sui'

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
export default controller<Scope>({
  scope: () => ({ rows: signal<SpaceRow[]>([]) }),

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
          host.dispatchEvent(new CustomEvent('sui-add', { bubbles: true, composed: true }))
          return
        }
        if (node.matches('.row[data-id]')) {
          const id = node.getAttribute('data-id')
          if (id && id !== host.getAttribute('active')) {
            host.dispatchEvent(
              new CustomEvent('sui-activate', { detail: { id }, bubbles: true, composed: true }),
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
