// Studio rail controller — URL-driven nav state for the Studio surface.
//
// The rail is a chrome component reused by every studio deck route. It has
// no view `route` signal of its own (custom elements don't receive the
// view mount ctx), so it tracks the active section by reading
// `location.pathname` and refreshing on popstate / sui:navigate — the same
// events the shell fires on every SPA navigation.
//
// The rail stays dependency-light inside the eager chrome bundle and renders
// honest "soon" placeholders (no href) for sections without a deck view yet.

import { controller, signal, derived, effect } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
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

// Components, themes, and pipelines are backed by real deck views and link
// out. MCP servers and permissions stay honest placeholders until their
// views land — no fabricated destinations.
const CONFIGURE: ReadonlyArray<RailEntry> = [
  { id: 'components', label: 'Components', icon: 'layers', href: '/studio/components' },
  { id: 'themes', label: 'Themes', icon: 'palette', href: '/studio/themes' },
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

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const path = signal(typeof location !== 'undefined' ? location.pathname : '/studio')

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

    const configureItems = derived<RailEntry[]>(() =>
      CONFIGURE.map((e) => ({
        ...e,
        active: e.href ? isActive(path(), e.href) : null,
      })),
    )

    return {
      projectItems: derived<RailEntry[]>(() => PROJECT.map((e) => ({ ...e }))),
      configureItems,
      libraryItems: derived<RailEntry[]>(() => LIBRARY.map((e) => ({ ...e }))),
    }
  },
})
