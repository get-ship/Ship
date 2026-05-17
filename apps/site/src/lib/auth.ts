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
 *
 * `plan` field
 * ────────────
 *   The gateway adds a `plan` additionalField (defaults to 'local') to
 *   `user`. We mirror the type here so the navbar and account/billing pages
 *   can render plan-aware UI without a second round-trip.
 */

import { createAuthClient } from 'better-auth/client';
import { stripeClient } from '@better-auth/stripe/client';

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
  plugins: [stripeClient({ subscription: true })],
});

export type Plan = 'local' | 'pro';

export interface User {
  id: string;
  name: string;
  email: string;
  image?: string | null;
  plan: Plan;
}

export interface Session {
  user: User;
  session: {
    id: string;
    token: string;
    expiresAt: string;
    createdAt: string;
    userAgent?: string | null;
    ipAddress?: string | null;
  };
}

/** Returns the current session or null. Always hits the network. */
export async function getSession(): Promise<Session | null> {
  try {
    const { data } = await authClient.getSession();
    if (!data?.user) return null;
    const u = data.user as Record<string, unknown>;
    return {
      user: {
        id: String(u.id),
        name: String(u.name ?? ''),
        email: String(u.email ?? ''),
        image: (u.image as string | null | undefined) ?? null,
        plan: (u.plan as Plan | undefined) ?? 'local',
      },
      session: data.session as Session['session'],
    };
  } catch {
    return null;
  }
}

export async function signOut(): Promise<void> {
  try {
    await authClient.signOut();
  } finally {
    emitChange();
  }
}

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
    const res = await authClient.forgetPassword({ email, redirectTo });
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

export async function listSessions() {
  try {
    const { data } = await authClient.listSessions();
    return data ?? [];
  } catch {
    return [];
  }
}

export async function listAccounts() {
  try {
    const { data } = await authClient.listAccounts();
    return data ?? [];
  } catch {
    return [];
  }
}

export async function unlinkAccount(providerId: string) {
  await authClient.unlinkAccount({ providerId });
}

// ── Stripe subscription wrappers ───────────────────────────────────────────
// Thin wrappers so pages call site-local helpers instead of poking the
// `subscription` namespace directly. Keeps the gateway swap surface small
// if the underlying API ever changes.

export type Subscription = {
  id: string;
  plan: string;
  status: string;
  periodStart?: string | Date;
  periodEnd?: string | Date;
  cancelAtPeriodEnd?: boolean;
  cancelAt?: string | Date;
  canceledAt?: string | Date;
  trialEnd?: string | Date;
  seats?: number;
  billingInterval?: string;
  stripeSubscriptionId?: string;
};

const DEFAULT_RETURN = (path: string) => `${location.origin}${path}`;

export async function startUpgrade(
  plan: 'pro',
  opts: { successPath?: string; cancelPath?: string; annual?: boolean } = {},
) {
  return authClient.subscription.upgrade({
    plan,
    annual: opts.annual ?? false,
    successUrl: DEFAULT_RETURN(opts.successPath ?? '/billing?upgrade=success'),
    cancelUrl: DEFAULT_RETURN(opts.cancelPath ?? '/pricing?upgrade=cancelled'),
  });
}

export async function openBillingPortal(returnPath = '/billing') {
  return authClient.subscription.billingPortal({
    returnUrl: DEFAULT_RETURN(returnPath),
  });
}

export async function listSubscriptions(): Promise<Subscription[]> {
  try {
    const { data } = await authClient.subscription.list();
    return (data ?? []) as Subscription[];
  } catch {
    return [];
  }
}

// `subscriptionId` here is the plugin's row id from the `subscription` table
// (the `id` column), NOT Stripe's `sub_…` id. The plugin uses it to look
// up the row and call Stripe with the right `stripeSubscriptionId` itself.
// Omitting works too — the plugin picks the user's active subscription —
// but being explicit avoids any ambiguity when there's history.
export async function cancelSubscription(subscriptionId: string, returnPath = '/billing') {
  return authClient.subscription.cancel({
    subscriptionId,
    returnUrl: DEFAULT_RETURN(returnPath),
  });
}

export async function restoreSubscription(subscriptionId: string) {
  return authClient.subscription.restore({ subscriptionId });
}

export async function revokeSession(token: string) {
  await authClient.revokeSession({ token });
}

export async function revokeOtherSessions() {
  await authClient.revokeOtherSessions();
}

export function initials(user: { name?: string; email: string }): string {
  if (user.name) {
    const parts = user.name.trim().split(/\s+/).slice(0, 2);
    return parts.map((p) => p[0]?.toUpperCase() ?? '').join('') || user.email[0]!.toUpperCase();
  }
  return user.email[0]!.toUpperCase();
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
