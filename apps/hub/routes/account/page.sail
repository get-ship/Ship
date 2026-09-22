<controller>
// Hub `/account` route controller — owner-level Ship Cloud identity.
//
// The shell's `/account` route was a non-functional placeholder (an
// `<sui-empty-state>` with a permanently-disabled "Connect account" button).
// The REAL account surface — the live state read plus sign-in / sign-out /
// billing — lived in the shell's settings card
// (`apps/shell/routes/settings`). This route is that real surface, promoted
// to its own page and rebuilt on the hub's ambient `ctx.ops` unary
// client.
//
// Contract: `ship.ops.v1.AccountService` (account.proto). All three ops used
// here are already on the unary surface — no proto/backend widening:
//   • account.state()  — snapshot { status, userId, me }            (read)
//   • account.login()  — PKCE start → returns a browser auth URL     (mutation)
//   • account.logout() — revoke gateway key + clear local token      (mutation)
//
// The unary `account.state` has no auto-refresh (the SSE live-resolver is its
// reactive twin, reserved for `ctx.ship`). A `reload` signal re-fires
// the read after a mutation, on a manual refresh, and when the tab regains
// focus — the last catches a sign-in that completed in the OAuth browser tab.
import { controller, signal, derived, live, unwrapOr } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'

// Mirrors `ship.ops.v1.AccountState.Status` (account.proto). The generated
// enum is not re-exported through `@ship/sdk/connect`, so the two non-default
// wire values are named here against the proto's documented numbering.
const STATUS_PENDING = 1
const STATUS_LOGGED_IN = 2

type Status = 'logged_out' | 'pending' | 'logged_in'
type MeLike = { name?: string; email?: string } | null

// PKCE redirect_uri is the daemon's local CLI callback; a browser SPA can't
// host one, so login hands the auth URL to the browser and the CLI owns the
// callback (matches `@ship/sdk-client`'s signIn helper + the chrome menu).
const LOGIN_REDIRECT_URI = 'http://127.0.0.1:0/callback'
const BILLING_URL = 'https://getship.dev/billing'

/** Two-letter monogram: name, then email local-part, else a neutral dash. */
function initialsOf(me: MeLike): string {
  const src = me?.name?.trim() || me?.email?.split('@')[0] || ''
  const parts = src.split(/[\s._-]+/).filter(Boolean)
  if (parts.length === 0) return '··'
  if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase()
  return (parts[0][0] + parts[1][0]).toUpperCase()
}

export default controller((ctx: ViewScope<Scope>) => {

    // AMBIENT client (host-owned): `entry.gen.ts` constructs `ops` from
    // `src/lib/client.ts` and injects it as `clients.ops`.
    const ops = ctx.ops
    const reload = signal(0)
    const busy = signal(false)
    const error = signal('')

    // Unary read; `reload()` is the only tracked dep, so bumping it (after a
    // mutation, a manual refresh, or a tab-focus) re-fires the read.
    const account = live(() => {
      reload()
      return ops.account.state({})
    })

    // `unwrapOr` collapses loading/error to the fallback — the controller
    // never branches on `.kind` (sui-controller-no-load-state); the template
    // owns the loading / error / ok render states off `account()`.
    const value = derived(() => unwrapOr(account, null))
    const status = derived<Status>(() => {
      const v = value()
      if (!v) return 'logged_out'
      if (v.status === STATUS_LOGGED_IN) return 'logged_in'
      if (v.status === STATUS_PENDING) return 'pending'
      return 'logged_out'
    })
    const me = derived(() => value()?.me ?? null)
    const name = derived(() => {
      const m = me()
      return m?.name?.trim() || m?.email?.trim() || value()?.userId || 'Ship Cloud account'
    })
    const email = derived(() => me()?.email?.trim() ?? '')
    const image = derived(() => me()?.image?.trim() ?? '')
    const initials = derived(() => initialsOf(me()))

    
    const signIn = async (s) => {
      if (s.busy) return
      s.busy = true
      s.error = ''
      try {
        const started = await s.clients.ops.account.login({ redirectUri: LOGIN_REDIRECT_URI })
        if (started.browserUrl) window.open(started.browserUrl, '_blank', 'noopener')
        // Flip to the pending state once the daemon has recorded the start.
        s.reload = s.reload + 1
      } catch (e) {
        s.error = e instanceof Error ? e.message : String(e)
      } finally {
        s.busy = false
      }
    }
    const signOut = async (s) => {
      if (s.busy) return
      s.busy = true
      s.error = ''
      try {
        await s.clients.ops.account.logout({})
        s.reload = s.reload + 1
      } catch (e) {
        s.error = e instanceof Error ? e.message : String(e)
      } finally {
        s.busy = false
      }
    }
    const refresh = (s) => {
      s.error = ''
      s.reload = s.reload + 1
    }
    const manageBilling = () => {
      window.open(BILLING_URL, '_blank', 'noopener')
    }
    return { account, status, name, email, image, initials, busy, error, reload, signIn, signOut, refresh, manageBilling }
})
</controller>

<!--
  Hub `/account` — owner-level Ship Cloud identity for this workspace.

  Replaces the shell's placeholder `/account` (a disabled "Connect account"
  button on an empty-state). The real surface — live state + sign-in /
  sign-out / billing — is rebuilt here on the unary `ctx.ops`
  account contract. State lives in `controller.ts`; this template only
  declares the four account states (loading / error / signed-in / pending /
  signed-out) it renders.

  Page chrome:
    • the top nav is owned by the root layout (`<ship-app-chrome>`)
    • page-level layout lives in the co-located `style.css`
      (deck-route-css convention) — never an inline <style>
    • tokens only — no hardcoded colors / spacing / radii
-->
<main data-view-enter data-test="account-view">
  <header>
    <h1>Account</h1>
    <p>Owner-level Ship Cloud identity for this workspace — sign-in, profile, and billing.</p>
  </header>

  <card padding="lg">
    <if cond={account().kind === 'loading'}>
      <div data-test="account-loading">
        <spinner size="md"></spinner>
        <span>Loading account…</span>
      </div>
    <else-if cond={account().kind === 'error'}>
      <empty-state icon="alert-triangle" variant="default" size="md" tone="muted" data-test="account-offline">
        <span slot="title">Ship Cloud unreachable</span>
        Account features are unavailable while the daemon is offline.
        <button slot="action" variant="outline" size="sm" on:click={refresh}>
          <icon slot="leading" name="refresh-cw" size="sm"></icon>
          Try again
        </button>
      </empty-state>
    </else-if><else-if cond={status() === 'logged_in'}>
      <div data-test="account-identity">
        <if cond={image()}>
          <img src={image()} alt="" />
        <else>
          <span aria-hidden="true">{initials()}</span>
        </else></if>
        <div>
          <span>{name()}</span>
          <if cond={email()}>
            <span>{email()}</span>
          <else>
            <span>Signed in to Ship Cloud</span>
          </else></if>
        </div>
        <badge tone="success" variant="soft" size="sm">Connected</badge>
      </div>

      <div>
        <div>
          <div>
            <span>Billing</span>
            <span>Manage your plan and payment method on Ship Cloud.</span>
          </div>
          <button variant="outline" size="sm" on:click={manageBilling} data-test="account-billing">
            <icon slot="leading" name="external-link" size="sm"></icon>
            Manage billing
          </button>
        </div>
        <div>
          <div>
            <span>Session</span>
            <span>Sign out clears the API key stored by this daemon.</span>
          </div>
          <button variant="outline" tone="destructive" size="sm"
            disabled={busy()} on:click={signOut} data-test="account-signout">
            <icon slot="leading" name="log-out" size="sm"></icon>
            Sign out
          </button>
        </div>
      </div>
    </else-if><else-if cond={status() === 'pending'}>
      <empty-state icon="loader" variant="connecting" size="md" tone="primary" data-test="account-pending">
        <span slot="title">Completing sign-in…</span>
        Finish the OAuth flow in the browser tab that opened, then return here. If the daemon's local
        callback doesn't complete, run <code>ship login</code> in a terminal to finalize.
        <button slot="action" variant="outline" size="sm" disabled={busy()} on:click={refresh}>
          <icon slot="leading" name="refresh-cw" size="sm"></icon>
          Check again
        </button>
      </empty-state>
    </else-if><else>
      <empty-state icon="user" variant="default" size="lg" tone="default" data-test="account-signedout">
        <span slot="title">Not signed in</span>
        Connect this workspace to Ship Cloud to unlock publish and sync.
        <button slot="action" tone="primary" variant="solid"
          disabled={busy()} on:click={signIn} data-test="account-signin">
          <icon slot="leading" name="log-in"></icon>
          Sign in with Ship Cloud
        </button>
      </empty-state>
    </else></if>

    <if cond={error()}>
      <alert tone="destructive" data-test="account-error">{error()}</alert>
    </if>
  </card>
</main>
