<controller>
// <ship-nav-top> — @ship/chrome · the filled top-bar controller.
//
// Sibling of the 3-slot <sui-nav-top> layout primitive in
// @ship/sail-primitives; that primitive is a generic bar with named
// slots. This one is opinionated: brand mark, breadcrumb trail, view
// actions, command-bar slot, theme picker, account menu, fullscreen.
//
// The account menu is the @ship/sail-primitives <sui-account-menu> — SDK-FREE
// by design. This controller owns its data wire: it subscribes to
// `ship.account.state()` (via the <data> element in the template) and flattens
// the live identity into the menu's status/name/email/plan props, and it turns
// the menu's `sui-account-*` events into the real SDK calls (@ship/sdk-client)
// + router navigation. The OG SDK-coupled chrome <sui-account-menu> was deleted
// in the chrome dissolution; that wiring lives HERE now (the host owns runtime).

import { controller, derived, signal, unwrapOr } from '@ship/sail-runtime'
import { navigate } from '@ship/deck-router'
import {
  accountDisplayName,
  accountSignIn,
  accountSignOut,
  openAccountBilling,
  readAccountIdentity,
  type AccountIdentity,
} from '@ship/sdk-client'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './ship-nav-top.sail'

/** Map the SDK identity status to the <sui-account-menu> prop vocabulary
 *  (logged_in → signed-in, pending → pending, else signed-out). */
function accountStatusOf(id: AccountIdentity): 'signed-in' | 'signed-out' | 'pending' {
  // `status` is narrowed to `'logged_out' | 'logged_in'` by the frozen
  // @ship/sdk-client@0.1.0 type, but the daemon (and the client's own
  // account.ts) emits a transient `'pending'` during pairing. Widen to
  // string for the comparison so the pending arm stays reachable.
  const status: string = id.status
  if (status === 'logged_in') return 'signed-in'
  if (status === 'pending') return 'pending'
  return 'signed-out'
}

interface Crumb {
  href: string
  label: string
  isLast: boolean
  // The keyed iteration in the template includes `isLast` in the key so
  // SUI doesn't reuse a previous crumb's branch (current-vs-link) when
  // the user navigates to a deeper route. Pure-`href` keys re-use a
  // crumb that flipped from leaf to mid-path and the runtime keeps the
  // old `{#if}` branch mounted, leaving the breadcrumb missing its
  // chevron + link affordance on the formerly-leaf segment.
  key: string
}

function humanize(id: string): string {
  return id
    .replace(/-/g, ' ')
    .split(' ')
    .map((w) => (w ? w[0]!.toUpperCase() + w.slice(1) : w))
    .join(' ')
}

async function fetchViewLabels(): Promise<Map<string, string>> {
  // Views are client-side routes; the daemon discovery catalog was quarantined
  // (06153c1a5). Cut the dead op — rail labels fall back to humanize() until the
  // router→view-list integration lands.
  return new Map<string, string>()
}

export default controller({
  scope: (state: ViewScope<Scope>) => {
    const path = state.path
    const projectProp = state.project
    const viewProp = state.view
    const labels = signal<Map<string, string>>(new Map())

    const crumbs = derived<Crumb[]>(() => {
      const labelsMap = labels()
      const project = projectProp().trim()
      const view = viewProp().trim()

      // Source segments: explicit `view` prop overrides path entirely.
      // Otherwise split the path; empty path falls back to the "dashboard"
      // crumb (parity with ship-nav-top.renderCrumbs).
      const segs: string[] = view
        ? [view]
        : path().split('/').filter(Boolean)
      // Home (`/`) shows no view crumb — the brand logo is the home button.

      const out: Crumb[] = []
      if (project) {
        // Project segment links to root; never the leaf (a view always
        // follows). Matches the ship-shell home-link convention.
        out.push({ href: '/', label: project, isLast: false, key: '/|0' })
      }
      segs.forEach((seg, i) => {
        const isLast = i === segs.length - 1
        const href = '/' + segs.slice(0, i + 1).join('/')
        const label = i === 0 ? (labelsMap.get(seg) ?? humanize(seg)) : seg
        out.push({ href, label, isLast, key: `${href}|${isLast ? 1 : 0}` })
      })
      return out
    })


    function onCrumbClick(e: MouseEvent, href: string): void {
      e.preventDefault()
      navigate(href)
    }
    function onBrand(e: MouseEvent): void {
      e.preventDefault()
      navigate('/')
    }
    function onFullscreen(): void {
      if (document.fullscreenElement) void document.exitFullscreen()
      else void document.documentElement.requestFullscreen()
    }

    // ── Account menu wire (the SDK-free <sui-account-menu> primitive) ────────
    // `account` is the live `ship.account.state()` envelope from the <data>
    // element. Flatten it into the menu's props; the load states (loading /
    // error) stay render concerns owned by the live source — we read a flat
    // identity via unwrapOr(..., null) (→ signed-out until the first emit).
    const account = state.account
    const accountBusy = signal(false)
    const accountError = signal('')
    const accountIdentity = derived<AccountIdentity>(() =>
      readAccountIdentity(unwrapOr(account, null)),
    )
    const accountStatus = derived(() => accountStatusOf(accountIdentity()))
    const accountName = derived(() => accountDisplayName(accountIdentity()))
    const accountEmail = derived(() => accountIdentity().me?.email ?? '')

    // Each action does the real work the SDK-free primitive can't: kick off the
    // PKCE login, clear the token, route to settings, open billing. Errors land
    // on the menu's `error` prop; `busy` disables the in-flight action.
    async function onAccountSignIn(): Promise<void> {
      if (accountBusy()) return
      accountBusy.set(true)
      accountError.set('')
      try {
        await accountSignIn()
      } catch (e) {
        accountError.set(e instanceof Error ? e.message : String(e))
      } finally {
        accountBusy.set(false)
      }
    }
    async function onAccountSignOut(): Promise<void> {
      if (accountBusy()) return
      accountBusy.set(true)
      accountError.set('')
      try {
        await accountSignOut()
      } catch (e) {
        accountError.set(e instanceof Error ? e.message : String(e))
      } finally {
        accountBusy.set(false)
      }
    }
    function onAccountSettings(): void {
      navigate('/settings#account')
    }
    function onAccountBilling(): void {
      openAccountBilling()
    }

    return {
      labels, crumbs,
      onCrumbClick, onBrand, onFullscreen,
      accountStatus, accountName, accountEmail, accountBusy, accountError,
      onAccountSignIn, onAccountSignOut, onAccountSettings, onAccountBilling,
    }
  },

  mount(state, host) {
    const pathSig = state.$signals.path
    const labelsSig = state.$signals.labels

    if (!pathSig()) pathSig.set(location.pathname)

    void fetchViewLabels().then((map) => labelsSig.set(map))

    const onNavigate = (e: Event) => {
      const to = (e as CustomEvent<{ to?: string }>).detail?.to
      if (typeof to === 'string') pathSig.set(to)
    }
    document.addEventListener('sui:navigate', onNavigate)

    host.dispatchEvent(
      new CustomEvent('sui:nav-top:ready', { bubbles: true, composed: true }),
    )

    return () => {
      document.removeEventListener('sui:navigate', onNavigate)
    }
  },
})
</controller>

<!--
  <ship-nav-top> — @ship/chrome · the filled top bar.

  COMPOSES the <navbar> primitive (@ship/sail-primitives) — it does NOT
  re-implement the bar. The primitive owns the slotted-bar structure (height,
  border, background, and the brand / default / actions slot regions); this
  component only assembles the ship-specific slot contents:

    brand   → home-linked <brand-mark>
    default → breadcrumb trail, per-view action slot, command-bar slot
    actions → theme picker, account menu, fullscreen toggle

  The three-slot layout primitive stays under <navbar>; this is the
  filled shell bar. <ship-nav-top> exposes its own pass-through slots
  (`actions`, `command-bar`) for the mounted view — these are forwarded into
  the primitive's default region.

  Prop / event surface
  ─────────────────────────────────────────────────────────────────────────────
  Props:
    path    (string, '')  current navigation path — auto-updated on sui:navigate
    project (string, '')  optional project label — when set, prepends a leading
                          home-linked breadcrumb segment (e.g. "Ship")
    view    (string, '')  optional explicit view name — when set, overrides the
                          path-derived root segment (humanized + label lookup
                          still apply). Lets the bar render correctly in
                          standalone harnesses where location.pathname is "/"
  ─────────────────────────────────────────────────────────────────────────────
  Events dispatched (bubbling, composed):
    sui:nav-top:ready {}  — fired once after first mount
  ─────────────────────────────────────────────────────────────────────────────
  Listens on document:
    sui:navigate  { detail: { to: string } }  — updates path + crumbs
  ─────────────────────────────────────────────────────────────────────────────
  Slots (pass-through into the primitive's default region):
    actions      per-view action buttons injected by the mounted view
    command-bar  command palette host — the host slots <ship-command-bar> here
  ─────────────────────────────────────────────────────────────────────────────
  Parts: brand, crumbs, theme-picker, account-menu,
         fullscreen
  ─────────────────────────────────────────────────────────────────────────────
  CSS custom properties:
    --ship-top-bar-h (default 40px)  bar height; forwarded to the primitive's
                                     ship-nav-top via the shared token
  ─────────────────────────────────────────────────────────────────────────────
  Navigation: `navigate()` from @ship/deck-router — the chrome pack has no
  chrome import. The single source of navigation lives in the route module.

  Dock affordance: the shell previously polled `body.dock-open` via a
  MutationObserver to mirror the panel dock state. The dock itself is
  not ported to a chrome-pack component yet, and there is no
  `ship.dock.state()` SDK surface to source from, so the
  MutationObserver is dropped for v1. When the dock lands, reintroduce
  detection as `<data name="dockOpen" source="ship.dock.state()" live />`.
  ─────────────────────────────────────────────────────────────────────────────
-->

<!-- <prop name="path" type="string" default="" /> -->
<!-- <prop name="project" type="string" default="" /> -->
<!-- <prop name="view" type="string" default="" /> -->

<!--
  The bar OWNS the account-state wire for the composed (SDK-free)
  <account-menu> primitive: the subscription lives in <data> per SUI hard
  rule #2 (never call ship.* outside <data>/<form mutation>). The controller
  flattens `account` into the menu's status/name/email/plan props and turns the
  primitive's sui-account-* events into the actual SDK calls + navigation.
-->
<client name="ship" />
<data name="account" source="ship.account.state()" live />

<!-- style block removed: use sx / external CSS -->

<navbar>
  <a slot="brand" href="/" aria-label="Ship home" title="Ship home" on:click={onBrand}>
    <brand-mark></brand-mark>
  </a>

  <nav aria-label="Breadcrumbs">
    <for as="c" of={crumbs()} key={c.key}>
      <if cond={c.isLast}>
        <span id="breadcrumbs-title" transition:fade={{ duration: 120 }}>{c.label}</span>
      <else>
        <a href={c.href} on:click={(e) => onCrumbClick(e, c.href)} transition:fade={{ duration: 120 }}>{c.label}</a>
        <span aria-hidden="true">›</span>
      </else></if>
    </for>
  </nav>
  <span><slot name="actions"></slot></span>
  <span></span>
  <span><slot name="command-bar"></slot></span>

  <theme-picker slot="actions" placement="bottom-end"></theme-picker>
  <account-menu slot="actions" placement="bottom-end"
    status={accountStatus()}
    name={accountName()}
    email={accountEmail()}
    busy={accountBusy()}
    error={accountError()}
    on:sail-account-signin={onAccountSignIn}
    on:sail-account-signout={onAccountSignOut}
    on:sail-account-settings={onAccountSettings}
    on:sail-account-billing={onAccountBilling}></account-menu>
  <button slot="actions" variant="ghost" size="sm"
    title="Fullscreen (F11)" aria-label="Toggle fullscreen" on:click={onFullscreen}>
    <icon slot="leading" name="maximize"></icon>
  </button>
</navbar>
