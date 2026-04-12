// POST /api/auth/daemon-attest — issue a signed JWT the local Ship daemon
// can verify to admit this browser session without CLI pairing.
//
// Authenticated via Better Auth session cookie (requireSession).
// Returns { jwt, issuer, expires_at } for the caller to forward to their daemon.

import { createFileRoute } from '@tanstack/react-router'
import { requireSession } from '#/lib/session-auth'
import { signDaemonJwt } from '#/lib/daemon-jwt'
import { getAuthDb } from '#/lib/d1'

function getIssuer(request: Request): string {
  const url = new URL(request.url)
  return `${url.protocol}//${url.host}`
}

export const Route = createFileRoute('/api/auth/daemon-attest')({
  server: {
    handlers: {
      POST: async ({ request }) => {
        const auth = await requireSession(request)
        if (auth instanceof Response) return auth

        // Look up the user's plan/caps so the daemon can enforce them.
        // For now we treat everyone as "free" until plan tracking is wired up.
        const db = getAuthDb()
        let plan: string | undefined
        if (db) {
          try {
            const row = await db
              .prepare('SELECT plan FROM user WHERE id = ? LIMIT 1')
              .bind(auth.sub)
              .first<{ plan?: string }>()
            plan = row?.plan ?? undefined
          } catch {
            // Optional column; missing `plan` is not fatal.
          }
        }

        const caps: string[] = plan === 'pro' ? ['pro'] : []
        const issuer = getIssuer(request)
        const ttlSeconds = 3600

        let jwt: string
        try {
          jwt = await signDaemonJwt({
            issuer,
            userId: auth.sub,
            plan,
            caps,
            ttlSeconds,
          })
        } catch (e) {
          return Response.json(
            { error: 'signing unavailable', detail: String(e) },
            { status: 503 },
          )
        }

        return Response.json({
          jwt,
          issuer,
          expires_at: Math.floor(Date.now() / 1000) + ttlSeconds,
        })
      },
    },
  },
})
