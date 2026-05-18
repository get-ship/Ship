import { Hono } from 'hono'
import { checkRateLimit, rateLimitResponse } from '../lib/rate-limit.js'

/**
 * Email capture — pre-launch waitlist + post-launch "notify me when X ships".
 *
 * Storage: Cloudflare KV (binding name WAITLIST). We key by a SHA-256 of the
 * lowercased email so collisions dedupe naturally and we don't accidentally
 * leak the address list to anyone with KV admin access.
 *
 * Anti-enumeration: we always return 200 regardless of whether the email
 * was new or already on the list. Probes can't tell if an address is
 * subscribed.
 *
 * No email sent yet — the value comes from having the list at launch.
 */
const waitlist = new Hono<{ Bindings: Env }>()

const ALLOWED_LISTS = new Set(['general', 'pro', 'changelog', 'blog'])

async function hashEmail(email: string): Promise<string> {
  const bytes = new TextEncoder().encode(email.trim().toLowerCase())
  const hash = await crypto.subtle.digest('SHA-256', bytes)
  return Array.from(new Uint8Array(hash), (b) => b.toString(16).padStart(2, '0')).join('')
}

function looksLikeEmail(s: unknown): s is string {
  if (typeof s !== 'string') return false
  if (s.length < 3 || s.length > 254) return false
  // Single, strict-enough check. Better Auth has its own validation on signup.
  return /^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(s)
}

waitlist.post('/v1/waitlist', async (c) => {
  const rl = await checkRateLimit(c.req.raw, 'RATE_LIMITER_CLAIM', 60, c.env)
  if (!rl.allowed) return rateLimitResponse(rl.retryAfter)

  const kv = (c.env as unknown as { WAITLIST?: KVNamespace }).WAITLIST
  if (!kv) return c.json({ error: 'waitlist_unavailable' }, 503)

  let body: unknown
  try { body = await c.req.json() } catch { return c.json({ error: 'invalid_json' }, 400) }
  const b = (body ?? {}) as Record<string, unknown>

  if (!looksLikeEmail(b.email)) return c.json({ error: 'invalid_email' }, 400)
  const email = (b.email as string).trim().toLowerCase()
  const list = typeof b.list === 'string' && ALLOWED_LISTS.has(b.list) ? (b.list as string) : 'general'
  const source = typeof b.source === 'string' ? b.source.slice(0, 64) : null

  const key = `waitlist:${list}:${await hashEmail(email)}`
  const existing = await kv.get(key, 'json')
  if (!existing) {
    await kv.put(
      key,
      JSON.stringify({ email, list, source, createdAt: Date.now() }),
    )
  }

  // Always 200 — anti-enumeration.
  return c.json({ ok: true })
})

export { waitlist as waitlistRoutes }
