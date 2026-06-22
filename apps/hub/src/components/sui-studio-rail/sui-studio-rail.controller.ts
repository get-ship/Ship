// Studio rail controller — URL-driven nav state for the Studio surface.
// (Hub port of `apps/shell/src/components/sui-studio-rail/
// sui-studio-rail.controller.ts`. Body is byte-identical to the shell
// except for two intentional changes:
//   1. The import seam — the shell pulls `ops` from its `../../ops`
//      wrapper, the hub has no `ops.ts` so it pulls the same
//      `createShipConnectClient` singleton from `../../lib/client`.
//   2. The listeners + count fetch moved out of the `scope:` block's
//      `effect()` bodies and into a `mount:` block with an explicit
//      cleanup return. The pre-fix shape (`effect(() => { addEventListener;
//      return () => removeEventListener })`) leaks listeners across every
//      studio nav: the chrome chain is rebuilt on every navigation
//      (mountRouter → buildMountTarget), the studio chrome's mount
//      runs a fresh rail, and the new rail's effect() has no tracked
//      deps so the effect's internal cleanup (the only path that
//      removes the listeners) never runs. `mount(): return cleanup`
//      is wired to `cleanups[]` by the runtime's mount and fires on
//      `handle.dispose()`.

import { controller, signal, derived } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-studio-rail.sui'
import { ops } from '../../lib/client'

interface RailEntry {
  id: string
  label: string
  icon: string
  /** Present only for sections backed by a real deck view. */
  href?: string
  /** `true` when the current path is under this section, else `null`
   *  (the boolean-attribute convention bindAttr expects). */
  active?: true | null
  /** Rendered count badge; empty string hides it. */
  count?: string
}

// Live routes: agents, skills, components. Collections and themes have
// hrefs but routes are pending — the router clears the content area on
// navigation while the chrome stays (chrome is fixed, lives above the
// router's replaceChildren scope).
const CONFIGURE: ReadonlyArray<RailEntry> = [
  { id: 'agents', label: 'Agents', icon: 'bot', href: '/studio/agents' },
  { id: 'skills', label: 'Skills', icon: 'zap', href: '/studio/skills' },
  { id: 'components', label: 'Components', icon: 'layers', href: '/studio/components' },
  { id: 'collections', label: 'Collections', icon: 'database', href: '/studio/collections' },
  { id: 'themes', label: 'Themes', icon: 'palette', href: '/studio/themes' },
]

function isActive(path: string, href: string): true | null {
  return path === href || path.startsWith(href + '/') ? true : null
}

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const path = signal(typeof location !== 'undefined' ? location.pathname : '/studio')
    const agentCount = signal<string>('')

    const configureItems = derived<RailEntry[]>(() =>
      CONFIGURE.map((e) => ({
        ...e,
        active: e.href ? isActive(path(), e.href) : null,
        count: e.id === 'agents' ? agentCount() : '',
      })),
    )

    return {
      configureItems,
      // Surface the path signal + count signal (raw refs, not the
      // proxied writable view) so the `mount:` block (which owns
      // teardown) can wire and unwire the listeners through the
      // standard lifecycle disposer instead of via an `effect()`
      // whose returned cleanup never fires. Pre-fix, the effect
      // had no tracked dependencies so it never re-ran; the
      // internal cleanup (which removes the listeners) only fires
      // on `effect.stop()`, which the scope never called. Every
      // mount of the studio chrome leaked a `popstate` +
      // `sui:navigate` listener pair, and the chrome is rebuilt
      // per navigation (mountRouter → buildMountTarget rebuilds
      // the chrome chain on every nav).
      path,
      agentCount,
    }
  },
  mount(state, _host) {
    // URL → path signal. One-shot per mount; the chrome persists
    // for the route's lifetime and re-mounts cleanly on navigation.
    // The returned disposer is the canonical teardown path —
    // `mount()`'s return value is wired to `cleanups[]` by
    // `@ship/sui-runtime`'s mount (mount.ts:276-278) and fires on
    // `handle.dispose()`. See ship-nav-top.controller.ts:108-120
    const onNav = (): void => { state.path = location.pathname }
    window.addEventListener('popstate', onNav)
    window.addEventListener('sui:navigate', onNav)

    // One-shot count fetch. Tolerant of shape + failure — a missing
    // count simply hides the badge. An AbortController cancels the
    // response write if the chrome unmounts before it settles, so
    // the post-dispose write never reaches a stale signal.
    const ac = new AbortController()
    void ops.agents
      .list({})
      .then((res) => {
        if (ac.signal.aborted) return
        state.agentCount = String(res.items.length)
      })
      .catch(() => { /* leave the badge empty when the API is unreachable */ })

    return () => {
      window.removeEventListener('popstate', onNav)
      window.removeEventListener('sui:navigate', onNav)
      ac.abort()
    }
  },
})
