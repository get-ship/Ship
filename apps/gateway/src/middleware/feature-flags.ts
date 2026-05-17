import type { Context, Next } from 'hono'

/**
 * Env-var-driven kill switches for cloud features.
 *
 * Default posture: cloud is OFF unless SHIP_CLOUD_ENABLED is explicitly set
 * to "true". This lets us deploy cloud routes in a pre-launch state without
 * surprising anyone who hits them — they get 503 with an honest reason.
 *
 * Routes that should NEVER be gated:
 *   - /healthz, /readyz, /api/version  (ops)
 *   - /api/auth/*                       (Better Auth: sign-in works without cloud)
 *   - /api/waitlist                     (pre-launch email capture)
 *   - /api/registry/*                   (registry browse + publish — pre-cloud)
 *
 * Everything else under /api/* should be wrapped by `requireFlag('cloud')`.
 */

export type FlagName = 'cloud' | 'stripe_live' | 'email_live' | 'macaroons'

export function flagEnabled(env: Env, flag: FlagName): boolean {
  const e = env as unknown as Record<string, string | undefined>
  switch (flag) {
    case 'cloud':       return e.SHIP_CLOUD_ENABLED === 'true'
    case 'stripe_live': return e.STRIPE_LIVE_MODE === 'true'
    case 'email_live':  return e.EMAIL_LIVE_MODE === 'true'
    case 'macaroons':   return e.MACAROON_ENFORCEMENT === 'true'
  }
}

export function requireFlag(flag: FlagName) {
  return async (c: Context<{ Bindings: Env }>, next: Next) => {
    if (flagEnabled(c.env, flag)) return next()
    return c.json(
      {
        error: 'feature_unavailable',
        flag,
        message: 'This feature is currently disabled. Try again later.',
      },
      503,
    )
  }
}
