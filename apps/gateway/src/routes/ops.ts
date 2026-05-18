import { Hono } from 'hono'

/**
 * Ops endpoints — always-on, no auth, no feature flag.
 *
 *   GET /healthz     liveness — returns 200 unconditionally
 *   GET /readyz      readiness — verifies critical bindings are reachable
 *   GET /v1/version  build identification
 *
 * Cloudflare's edge needs liveness probes that don't touch backing services
 * (avoid cascading failure during D1 incidents). Readiness DOES touch them,
 * so it surfaces partial degradation to load balancers.
 */
const ops = new Hono<{ Bindings: Env }>()

ops.get('/healthz', (c) => c.text('ok', 200))

ops.get('/readyz', async (c) => {
  const checks: Record<string, 'ok' | 'unavailable' | 'unconfigured'> = {}

  // D1 — auth DB is required; registry DB is required.
  for (const [name, binding] of Object.entries({
    auth_db: c.env.AUTH_DB,
    registry_db: c.env.REGISTRY_DB,
  })) {
    if (!binding) {
      checks[name] = 'unconfigured'
      continue
    }
    try {
      await (binding as D1Database).prepare('SELECT 1').first()
      checks[name] = 'ok'
    } catch {
      checks[name] = 'unavailable'
    }
  }

  // KV — only required if we expect waitlist/etc, but checking when bound.
  const kv = (c.env as unknown as { WAITLIST?: KVNamespace }).WAITLIST
  if (kv) {
    try {
      await kv.get('__readyz__')
      checks.waitlist_kv = 'ok'
    } catch {
      checks.waitlist_kv = 'unavailable'
    }
  }

  const allOk = Object.values(checks).every((v) => v === 'ok' || v === 'unconfigured')
  return c.json(
    { status: allOk ? 'ok' : 'degraded', checks },
    allOk ? 200 : 503,
  )
})

ops.get('/v1/version', (c) => {
  // Build SHA and version come from wrangler-injected env at deploy time
  // (or from a build-time replacement). Until that's wired, the version
  // string is "dev"; build SHA is whatever last `wrangler deploy` stamped,
  // available via Cloudflare's `cf-ray` only — surface 'unknown' otherwise.
  const env = c.env as unknown as { GATEWAY_VERSION?: string; BUILD_SHA?: string }
  return c.json({
    gateway: env.GATEWAY_VERSION ?? 'dev',
    build_sha: env.BUILD_SHA ?? 'unknown',
  })
})

export { ops as opsRoutes }
