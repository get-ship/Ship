// Studio rail controller — URL-driven nav state for the Studio surface.
//
// The rail is a chrome component reused by every studio deck route. It has
// no view `route` signal of its own (custom elements don't receive the
// view mount ctx), so it tracks the active section by reading
// `location.pathname` and refreshing on popstate / sui:navigate — the same
// events the shell fires on every SPA navigation.
//
// Counts come from the daemon REST API, not the SDK live channel, so the
// rail stays dependency-light inside the eager chrome bundle. Only the
// Agents section is backed by a deck view today; the rest are honest
// "soon" placeholders (no href, no fabricated count).

import { controller, signal, derived, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-studio-rail.sui'

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

const PROJECT: ReadonlyArray<RailEntry> = [
  { id: 'overview', label: 'Overview', icon: 'circle' },
  { id: 'isolation', label: 'Isolation', icon: 'box' },
]

// Agents is the one section live as a deck view (views/studio/agents/**).
// The others land as deck views in follow-ups; until then they're honest
// placeholders rather than links into the void.
const CONFIGURE: ReadonlyArray<RailEntry> = [
  { id: 'agents', label: 'Agents', icon: 'bot', href: '/studio/agents' },
  { id: 'skills', label: 'Skills', icon: 'zap', href: '/studio/skills' },
  { id: 'components', label: 'Components', icon: 'layers', href: '/studio/components' },
  { id: 'themes', label: 'Themes', icon: 'palette', href: '/studio/themes' },
  { id: 'collections', label: 'Collections', icon: 'database', href: '/studio/collections' },
  { id: 'pipelines', label: 'Pipelines', icon: 'workflow', href: '/studio/pipelines' },
  { id: 'mcp', label: 'MCP servers', icon: 'terminal' },
  { id: 'permissions', label: 'Permissions', icon: 'shield' },
]

const LIBRARY: ReadonlyArray<RailEntry> = [
  { id: 'installed', label: 'Installed', icon: 'package' },
  { id: 'marketplace', label: 'Marketplace', icon: 'store' },
  { id: 'publishing', label: 'Publishing', icon: 'upload' },
]

function isActive(path: string, href: string): true | null {
  return path === href || path.startsWith(href + '/') ? true : null
}

export default controller<Scope>({
  scope: () => {
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
      if (typeof fetch === 'undefined') return
      let cancelled = false
      void fetch('/api/v1/agents', { credentials: 'same-origin' })
        .then((r) => (r.ok ? r.json() : null))
        .then((body) => {
          if (cancelled || !body) return
          const items = Array.isArray(body) ? body : (body.items ?? body.data ?? [])
          if (Array.isArray(items)) agentCount.set(String(items.length))
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
      projectItems: derived<RailEntry[]>(() => PROJECT.map((e) => ({ ...e }))),
      configureItems,
      libraryItems: derived<RailEntry[]>(() => LIBRARY.map((e) => ({ ...e }))),
    }
  },
})
