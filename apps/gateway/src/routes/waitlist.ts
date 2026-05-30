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

const ALLOWED_OS = new Set(['macos', 'windows', 'linux', 'wsl', 'other'])
const ALLOWED_HARNESSES = new Set(['claude-code', 'codex', 'cursor', 'opencode', 'pi', 'none'])
const ALLOWED_CHANNELS = new Set(['x', 'hn', 'podcast', 'friend', 'docs-or-repo', 'search', 'other'])
const ALLOWED_SECURITY = new Set(['none', 'containers', 'vms', 'sandbox', 'not-sure'])

function pick<T>(value: unknown, allowed: Set<T>): T | null {
  return typeof value === 'string' && (allowed as Set<unknown>).has(value) ? (value as T) : null
}
function pickMany<T>(value: unknown, allowed: Set<T>, cap = 8): T[] {
  if (!Array.isArray(value)) return []
  const seen = new Set<T>()
  for (const v of value) {
    if (typeof v === 'string' && (allowed as Set<unknown>).has(v)) seen.add(v as T)
    if (seen.size >= cap) break
  }
  return Array.from(seen)
}

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

  // Survey fields — strict enum allow-lists so we don't end up with free-form
  // analytics. harness / channel / security are REQUIRED: we only want
  // signups with the questions captured, so an incomplete payload is rejected
  // before it can reach KV. (os is auto-detected client-side; left optional.)
  const os = pick(b.os, ALLOWED_OS)
  const harnesses = pickMany(b.harnesses, ALLOWED_HARNESSES, 8)
  const channel = pick(b.channel, ALLOWED_CHANNELS)
  const security = pick(b.security, ALLOWED_SECURITY)
  if (harnesses.length === 0 || !channel || !security) {
    return c.json({ error: 'survey_required' }, 400)
  }

  const key = `waitlist:${list}:${await hashEmail(email)}`
  const existing = await kv.get(key, 'json')
  if (!existing) {
    await kv.put(
      key,
      JSON.stringify({ email, list, source, os, harnesses, channel, security, createdAt: Date.now() }),
    )
  }

  // Always 200 — anti-enumeration.
  return c.json({ ok: true })
})

export { waitlist as waitlistRoutes }
