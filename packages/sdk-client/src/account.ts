// Account-action helpers shared between <sui-account-menu> (nav-top chrome)
// and the Account card in views/settings. Keeps the daemon REST calls + the
// browser-flow choreography in one place so the two surfaces don't drift.

import type { AccountState, MeView } from './generated'
import { createShipConnectClient } from '@ship/sdk/connect'

// Explicit union: the proto removed 'pending' from AccountState, but the
// daemon may still emit it during in-flight logins (PKCE state). Keep it
// here as a runtime-defensive variant so readIdentity's switch is not dead
// code while a server/client version skew exists.
export type AccountStatus = 'logged_out' | 'pending' | 'logged_in'

export interface AccountIdentity {
  status: AccountStatus
  user_id: string | null
  me: MeView | null
  state_id: string | null
}
// Wider local union that preserves the 'pending' variant the proto dropped.
// The daemon may still emit it during in-flight PKCE logins; this lets the
// discriminated-union switch below narrow correctly without an `any` cast.
type WireAccountState =
  | AccountState
  | { state: 'pending'; state_id: string }

/** Normalise the daemon's tagged-enum state into a flat shape that templates
 *  can read without re-doing the discriminant dance. `me` may be null even
 *  when logged in — the daemon's `state()` only populates `me` after a
 *  fresh login (no auto-refresh today). */
export function readIdentity(state: AccountState | null | undefined): AccountIdentity {
  if (!state) return { status: 'logged_out', user_id: null, me: null, state_id: null }
  const s = state as WireAccountState
  switch (s.state) {
    case 'logged_in':
      return { status: 'logged_in', user_id: s.user_id, me: s.me ?? null, state_id: null }
    case 'pending':
      return { status: 'pending', user_id: null, me: null, state_id: s.state_id }
    default:
      return { status: 'logged_out', user_id: null, me: null, state_id: null }
  }
}

/** Display name for the chrome trigger / settings card header. */
export function displayName(id: AccountIdentity): string {
  if (id.status !== 'logged_in') return ''
  const name = id.me?.name?.trim()
  if (name) return name
  const email = id.me?.email?.trim()
  if (email) return email
  return id.user_id ?? ''
}

/** Two-letter monogram for the avatar fallback. Prefers name, falls back to
 *  email local-part, then user_id. */
export function initials(id: AccountIdentity): string {
  if (id.status !== 'logged_in') return ''
  const source =
    id.me?.name?.trim() ||
    id.me?.email?.split('@')[0] ||
    id.user_id ||
    ''
  const parts = source.split(/[\s._-]+/).filter(Boolean)
  if (parts.length === 0) return ''
  if (parts.length === 1) return parts[0]!.slice(0, 2).toUpperCase()
  return (parts[0]![0]! + parts[1]![0]!).toUpperCase()
}

/** Plan label shown next to the identity. Returns empty string when the
 *  gateway hasn't surfaced a plan field (today's state — the daemon's
 *  `state()` doesn't include plan in `MeView` yet). Templates should
 *  guard with `{#if planLabel()}…{/if}` so the chip doesn't render a
 *  wrong/misleading label.
 *
 *  TODO: once the gateway propagates plan into the daemon-attest payload
 *  (and from there into MeView), swap to `id.me?.plan` and remove the
 *  empty-string fallback. Tracked alongside the `caps` propagation work
 *  in the auth follow-ups. */
export function planLabel(_id: AccountIdentity): string {
  // No `plan` field on MeView today; surfacing nothing is honest. Don't
  // guess. A future MeView with plan:string flips this to `id.me?.plan ?? ''`.
  return ''
}

interface StartLoginResponse {
  state_id: string
  browser_url: string
}

/** Kick off the PKCE login. POSTs to the daemon, opens the browser URL it
 *  returns in a new tab (the daemon's redirect_uri is the local CLI callback;
 *  Shell can't host one, so we hand it off to the browser). After the user
 *  completes the flow, the daemon's `/state` route flips to `logged_in` and
 *  the live signal re-emits. Returns the state_id so callers can show a
 *  "completing login…" indicator while it's pending. */
export async function signIn(redirectUri?: string): Promise<StartLoginResponse> {
  // Same-origin Connect: the daemon's auth middleware reads the
  // session cookie, so no token plumbing. The wire's `AccountService.Login`
  // returns a typed `LoginStarted { stateId, browserUrl }`; we hand
  // `browserUrl` off to the browser (the daemon's redirect_uri is the
  // local CLI callback — Shell can't host one, so the browser-side
  // `ship login` CLI owns the callback today).
  const connect = createShipConnectClient()
  const started = await connect.account.login({
    redirectUri: redirectUri ?? 'http://127.0.0.1:0/callback',
  })
  if (started.browserUrl) {
    window.open(started.browserUrl, '_blank', 'noopener')
  }
  return {
    state_id: started.stateId,
    browser_url: started.browserUrl,
  }
}

/** Clear the daemon's stored token. Idempotent on the daemon side. */
export async function signOut(): Promise<void> {
  // Same-origin Connect: the wire's `AccountService.Logout` returns a
  // typed `LogoutResult { hadToken, keyRevoked }`. Connect throws
  // `ConnectError` on failure (no `resp.ok` to check) — the previous
  // REST path surfaced 204 as success; the typed wire returns a
  // result either way, so a 204-equivalent "no token" case
  // short-circuits cleanly here.
  const connect = createShipConnectClient()
  await connect.account.logout({})
}

/** Open Ship Cloud's billing page in a new browser tab. The marketing site
 *  redirects through Better Auth → Stripe portal when the user is signed in. */
export function openBilling(): void {
  window.open('https://getship.dev/billing', '_blank', 'noopener')
}
