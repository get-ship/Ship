/**
 * Marketing-site auth — thin shim over Better Auth's client SDK pointed at
 * the Ship gateway worker (apps/gateway).
 *
 * Gateway URL resolution
 * ──────────────────────
 *   1. PUBLIC_GATEWAY_URL env var if set (escape hatch for local-gateway dev)
 *   2. import.meta.env.DEV → https://staging.getship.dev
 *   3. otherwise           → https://api.getship.dev
 *
 * Local dev caveat
 * ────────────────
 *   The session cookie is set on `.getship.dev` (see gateway's
 *   `advanced.crossSubDomainCookies`). A marketing site served from
 *   `localhost:4321` is a different domain entirely, so cookies set by the
 *   staging gateway will not be readable by the local browser tab. The OAuth
 *   redirect flow will still complete on the gateway side, but the
 *   marketing site won't see the session.
 *
 *   To exercise the full loop locally, run the gateway too:
 *
 *     cd apps/gateway && bun wrangler dev
 *     PUBLIC_GATEWAY_URL=http://localhost:8787 bun --filter @ship/site dev
 */

import { createAuthClient } from 'better-auth/client';

// Gateway URL resolution
// ──────────────────────
//   1. PUBLIC_GATEWAY_URL env var if set — escape hatch (e.g. point at
//      staging from a local dev session)
//   2. dev mode → http://localhost:8787 — assumes you're running the gateway
//      alongside via `cd apps/gateway && bun wrangler dev`
//   3. prod build → https://api.getship.dev
const GATEWAY_URL =
  (import.meta.env.PUBLIC_GATEWAY_URL as string | undefined) ??
  (import.meta.env.DEV ? 'http://localhost:8787' : 'https://api.getship.dev');

export const authClient = createAuthClient({
  baseURL: GATEWAY_URL,
  // Gateway mounts Better Auth at /v1/auth/* (apps/gateway/src/lib/auth.ts
  // sets basePath). Keep these in sync.
  basePath: '/v1/auth',
});

export async function signInWithProvider(
  provider: 'google' | 'github' | 'slack',
  callbackURL = '/account',
): Promise<void> {
  await authClient.signIn.social({ provider, callbackURL });
}

export async function signInWithEmail(
  email: string,
  password: string,
  callbackURL = '/account',
): Promise<{ ok: true } | { ok: false; error: string }> {
  const res = await authClient.signIn.email({ email, password, callbackURL });
  if (res.error) return { ok: false, error: humanize(res.error.message ?? res.error.code) };
  emitChange();
  return { ok: true };
}

export async function signUpWithEmail(
  name: string,
  email: string,
  password: string,
  callbackURL = '/account',
): Promise<{ ok: true } | { ok: false; error: string }> {
  const res = await authClient.signUp.email({ name, email, password, callbackURL });
  if (res.error) return { ok: false, error: humanize(res.error.message ?? res.error.code) };
  emitChange();
  return { ok: true };
}

export async function requestPasswordReset(
  email: string,
  redirectTo = '/reset-password',
): Promise<{ ok: true } | { ok: false; error: string }> {
  try {
    const res = await authClient.requestPasswordReset({ email, redirectTo });
    if (res?.error) return { ok: false, error: humanize(res.error.message ?? res.error.code) };
    return { ok: true };
  } catch (e) {
    return { ok: false, error: (e as Error).message ?? 'Could not request reset.' };
  }
}

export async function resetPassword(
  newPassword: string,
  token: string,
): Promise<{ ok: true } | { ok: false; error: string }> {
  try {
    const res = await authClient.resetPassword({ newPassword, token });
    if (res?.error) return { ok: false, error: humanize(res.error.message ?? res.error.code) };
    return { ok: true };
  } catch (e) {
    return { ok: false, error: (e as Error).message ?? 'Could not reset password.' };
  }
}

function humanize(msg: string | undefined): string {
  if (!msg) return 'Something went wrong. Please try again.';
  // Better Auth returns either i18n codes or human messages. Map the codes
  // we care about; pass through the rest.
  const m = msg.toLowerCase();
  if (m.includes('invalid') && m.includes('credentials')) return 'Email or password is incorrect.';
  if (m.includes('user') && m.includes('exists')) return 'An account already exists for that email.';
  if (m.includes('password') && m.includes('short')) return 'Password must be at least 8 characters.';
  return msg;
}

function emitChange(): void {
  if (typeof document === 'undefined' || typeof CustomEvent === 'undefined') return;
  document.dispatchEvent(new CustomEvent('ship:auth:change'));
}
