// Studio rail controller — URL-driven nav state for the Studio surface.
// (Hub port of `apps/shell/src/components/sui-studio-rail/
// sui-studio-rail.controller.ts`. Body is byte-identical to the shell;
// the only change is the import seam: the shell pulls `ops` from its
// `../../ops` wrapper, the hub has no `ops.ts` so it pulls the same
// `createShipConnectClient` singleton from `../../lib/client`.)
//
// The rail is a chrome component reused by every studio deck route. It has
// no view `route` signal of its own (custom elements don't receive the
// view mount ctx), so it tracks the active section by reading
// `location.pathname` and refreshing on popstate / sui:navigate — the same
// events the shell fires on every SPA navigation.
//
// Counts come from the daemon REST API, not the SDK live channel, so the
// rail stays dependency-light inside the eager chrome bundle. Agents,
// skills, and components have live deck routes. Collections and themes link
// out but their routes are not yet implemented; the router clears the route
// slot on an unmatched path.

import { controller, signal, derived, effect } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-studio-rail.sui'
import { ship } from '../../lib/client'

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

    effect(() => {
      if (typeof window === 'undefined') return
      const onNav = () => path.set(location.pathname)
      window.addEventListener('popstate', onNav)
      window.addEventListener('sui:navigate', onNav)
      return () => {
        window.removeEventListener('popstate', onNav)
        window.removeEventListener('sui:navigate', onNav)
      }
    })

    // One-shot count fetch. Tolerant of shape + failure — a missing count
    // simply hides the badge.
    effect(() => {
      let cancelled = false
      void ship.agents
        .list({})
        .then((res) => {
          if (cancelled) return
          agentCount.set(String(res.items.length))
        })
        .catch(() => { /* leave the badge empty when the API is unreachable */ })
      return () => { cancelled = true }
    })

    const configureItems = derived<RailEntry[]>(() =>
      CONFIGURE.map((e) => ({
        ...e,
        active: e.href ? isActive(path(), e.href) : null,
        count: e.id === 'agents' ? agentCount() : '',
      })),
    )

    return {
      configureItems,
    }
  },
})
