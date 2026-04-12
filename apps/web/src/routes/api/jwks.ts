// GET /api/jwks — public keys for daemon attestation JWT verification.
//
// The Ship daemon fetches this endpoint to verify JWTs signed by getship.dev.
// Contains the current public signing key and any rotating-out keys whose
// issued JWTs have not yet expired.

import { createFileRoute } from '@tanstack/react-router'
import { buildJwks } from '#/lib/daemon-jwt'

export const Route = createFileRoute('/api/jwks')({
  server: {
    handlers: {
      GET: async () => {
        try {
          const jwks = await buildJwks()
          return Response.json(jwks, {
            headers: {
              'Cache-Control': 'public, max-age=3600',
              'Content-Type': 'application/jwk-set+json',
            },
          })
        } catch (e) {
          return Response.json(
            { error: 'jwks unavailable', detail: String(e) },
            { status: 503 },
          )
        }
      },
    },
  },
})
