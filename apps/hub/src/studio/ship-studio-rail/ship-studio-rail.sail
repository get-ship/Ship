<controller>
// <ship-studio-rail> controller — @ship/studio · URL-driven nav state for
// the Studio surface.
//
// The listeners + agent-count fetch live in a `mount:` block with an
// explicit cleanup return (not in a `scope:` `effect()`). The chrome chain
// is rebuilt on every navigation (mountRouter → buildMountTarget), the
// studio chrome's mount runs a fresh rail, and a `scope:` effect with no
// tracked deps never re-runs — so its internal cleanup never fires and the
// listeners leak. `mount(): return cleanup` is wired to `cleanups[]` by the
// runtime's mount and fires on `handle.dispose()`.

import { controller, signal, derived } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import { createShipConnectClient } from '@ship/sdk/connect'
import type { Scope } from './ship-studio-rail.sail'

// Host-agnostic same-origin Connect client — cookies carry the session, so
// no token plumbing. The pack owns its data plumbing; the host wires nothing.
const ops = createShipConnectClient({ baseUrl: '' })

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
    // `@ship/sail-runtime`'s mount (mount.ts:276-278) and fires on
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
</controller>

<!--
  <ship-studio-rail> — @ship/studio · the Studio navigation rail.

  A host's studio routes (/studio/agents, /studio/agents/:id, …) each drop
  this rail in their layout. It resolves via the bundler's per-tag scan, so
  nested leaves can name `<ship-studio-rail>` without importing it.

  Navigation is plain deck: every entry is an `<a href="/studio/…">`,
  intercepted by the host's anchor delegate (which walks composedPath, so
  shadow-DOM links route as SPA navigations). Active state is read from
  `location.pathname`, refreshed on popstate / sui:navigate.

  Only sections backed by a real deck view link out. The rest stay honest
  "soon" placeholders until their view lands — no fabricated destinations,
  no 404s.
-->

<!-- style block removed: use sx / external CSS -->

<a href="/studio" data-test="rail-home">
  <span aria-hidden="true"><icon name="grid" size="sm"></icon></span>
  <span>Studio</span>
  <span aria-hidden="true"></span>
</a>

<nav aria-label="Configure">
  <div>Configure <span>{configureItems().length}</span></div>
  <for as="item" of={configureItems()} key={item.id}>
    <a data-test="rail-item"
      data-section={item.id}
      data-active={item.active}
      href={item.href}>
      <span aria-hidden="true"><icon name={item.icon} size="xs"></icon></span>
      <span>{item.label}</span>
      <if cond={item.count}><span>{item.count}</span></if>
    </a>
  </for>
</nav>

<div>
  <a data-test="rail-close" href="/">
    <icon name="chevron-left" size="xs"></icon>
    <span>Close Studio</span>
  </a>
</div>
