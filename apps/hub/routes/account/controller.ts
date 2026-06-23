// Hub `/account` route controller — owner-level Ship Cloud identity.
//
// The shell's `/account` route was a non-functional placeholder (an
// `<sui-empty-state>` with a permanently-disabled "Connect account" button).
// The REAL account surface — the live state read plus sign-in / sign-out /
// billing — lived in the shell's settings card
// (`apps/shell/routes/settings`). This route is that real surface, promoted
// to its own page and rebuilt on the hub's ambient `ctx.clients.ops` unary
// client.
//
// Contract: `ship.ops.v1.AccountService` (account.proto). All three ops used
// here are already on the unary surface — no proto/backend widening:
//   • account.state()  — snapshot { status, userId, me }            (read)
//   • account.login()  — PKCE start → returns a browser auth URL     (mutation)
//   • account.logout() — revoke gateway key + clear local token      (mutation)
//
// The unary `account.state` has no auto-refresh (the SSE live-resolver is its
// reactive twin, reserved for `ctx.clients.ship`). A `reload` signal re-fires
// the read after a mutation, on a manual refresh, and when the tab regains
// focus — the last catches a sign-in that completed in the OAuth browser tab.
import { controller, signal, derived, live, unwrapOr } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

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

export default controller({
  scope: (ctx: ViewScope<Scope>) => {
    // AMBIENT client (host-owned): `entry.gen.ts` constructs `ops` from
    // `src/lib/client.ts` and injects it as `clients.ops`.
    const ops = ctx.clients.ops
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

    return { account, status, name, email, image, initials, busy, error, reload }
  },

  on: {
    signIn: async (s) => {
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
    },
    signOut: async (s) => {
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
    },
    refresh: (s) => {
      s.error = ''
      s.reload = s.reload + 1
    },
    manageBilling: () => {
      window.open(BILLING_URL, '_blank', 'noopener')
    },
  },

  mount(state) {
    // Re-read when the tab regains focus — catches a sign-in completed in the
    // OAuth browser tab (the unary read has no SSE auto-refresh).
    const onVisible = (): void => {
      if (document.visibilityState === 'visible') state.reload = state.reload + 1
    }
    document.addEventListener('visibilitychange', onVisible)
    return () => document.removeEventListener('visibilitychange', onVisible)
  },
})
