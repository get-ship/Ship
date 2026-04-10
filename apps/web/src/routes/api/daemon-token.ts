import { createFileRoute } from '@tanstack/react-router'
import { DAEMON_BASE_URL } from '#/lib/daemon-config'

export const Route = createFileRoute('/api/daemon-token')({
  server: {
    handlers: {
      GET: async () => {
        // Proxy to daemon's handshake endpoint.
        // Falls back to reading token from env for local dev.
        try {
          // Try daemon handshake first (unauthenticated, localhost only)
          const res = await fetch(`${DAEMON_BASE_URL}/auth/handshake`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ capabilities: ['*'] }),
          })
          if (res.ok) {
            const body = await res.json() as { token: string }
            return Response.json({ token: body.token })
          }
        } catch {
          // Daemon not running or handshake not implemented
        }

        // Fallback: env var (set by container startup or dev scripts)
        const token = process.env.SHIP_DAEMON_TOKEN ?? null
        if (token) return Response.json({ token })

        return Response.json({ token: null }, { status: 404 })
      },
    },
  },
})
