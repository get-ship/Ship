<scope>
  props: {
    path = ''
    project = ''
    view = ''
  }
</scope>
<controller>
// FILE_CAP_OVERRIDE: filled top-bar owns crumb derivation + account-menu SDK wire in one controller.
import { controller, derived, mount, signal, unwrapOr } from '@ship/sail-runtime'
import { navigate } from '@ship/deck-router'
import {
  accountDisplayName,
  accountSignIn,
  accountSignOut,
  openAccountBilling,
  readAccountIdentity,
  type AccountIdentity,
} from '@ship/sdk-client'
import type { Scope } from './ship-nav-top.sail'

/** Map the SDK identity status to the <account-menu> prop vocabulary
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
  // the reconciler doesn't reuse a previous crumb's branch (current-vs-link)
  // when the user navigates to a deeper route.
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

export default controller((ctx: Scope) => {
  const labels = signal<Map<string, string>>(new Map())

  const crumbs = derived<Crumb[]>(() => {
    const labelsMap = labels()
    const project = ctx.project().trim()
    const view = ctx.view().trim()

    // Source segments: explicit `view` prop overrides path entirely.
    // Otherwise split the path; empty path falls back to the "dashboard"
    // crumb (parity with ship-nav-top.renderCrumbs).
    const segs: string[] = view
      ? [view]
      : ctx.path().split('/').filter(Boolean)
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

  const accountBusy = signal(false)
  const accountError = signal('')
  const accountIdentity = derived<AccountIdentity>(() =>
    readAccountIdentity(unwrapOr(ctx.account, null)),
  )
  const accountStatus = derived(() => accountStatusOf(accountIdentity()))
  const accountName = derived(() => accountDisplayName(accountIdentity()))
  const accountEmail = derived(() => accountIdentity().me?.email ?? '')

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

  mount((host) => {
    if (!ctx.path()) ctx.path.set(location.pathname)

    void fetchViewLabels().then((map) => labels.set(map))

    const onNavigate = (e: Event) => {
      const to = (e as CustomEvent<{ to?: string }>).detail?.to
      if (typeof to === 'string') ctx.path.set(to)
    }
    document.addEventListener('sui:navigate', onNavigate)

    host.dispatchEvent(
      new CustomEvent('sui:nav-top:ready', { bubbles: true, composed: true }),
    )

    return () => {
      document.removeEventListener('sui:navigate', onNavigate)
    }
  })

  return {
    labels, crumbs,
    onCrumbClick, onBrand, onFullscreen,
    accountStatus, accountName, accountEmail, accountBusy, accountError,
    onAccountSignIn, onAccountSignOut, onAccountSettings, onAccountBilling,
  }
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
    actions → theme picker, account menu, fullscreen

  The three-slot layout primitive stays under <navbar>; this is the
  filled shell bar. <ship-nav-top> exposes its own pass-through slots
  (`actions`, `command-bar`) for the mounted view — these are forwarded into
  the primitive's default region.

  Props:
    path    (string, '')  current navigation path — auto-updated on sui:navigate
    project (string, '')  optional project label — when set, prepends a leading
                          home-linked breadcrumb segment (e.g. "Ship")
    view    (string, '')  optional explicit view name — when set, overrides the
                          path-derived root segment (humanized + label lookup
                          still apply). Lets the bar render correctly in
                          standalone harnesses where location.pathname is "/"
  Events dispatched (bubbling, composed):
    sui:nav-top:ready {}  — fired once after first mount
  Listens on document:
    sui:navigate  { detail: { to: string } }  — updates path + crumbs
-->

<client name="ship" />
<data name="account" source="ship.account.state()" live />

<navbar>
  <a slot="brand" href="/" aria-label="Ship home" title="Ship home" on:click={onBrand}>
    <brand-mark></brand-mark>
  </a>

  <nav aria-label="Breadcrumbs">
    <for {crumbs() as c} key={c.key}>
      <if {c.isLast}>
        <span id="breadcrumbs-title" transition:fade={{ duration: 120 }}>{c.label}</span>
      {else =>
        <a href={c.href} on:click={(e) => onCrumbClick(e, c.href)} transition:fade={{ duration: 120 }}>{c.label}</a>
        <span aria-hidden="true">›</span>
      }</if>
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
