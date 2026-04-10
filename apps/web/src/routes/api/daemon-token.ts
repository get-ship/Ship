import { createFileRoute } from '@tanstack/react-router'
import { readDaemonToken } from '#/lib/daemon-token'

export const Route = createFileRoute('/api/daemon-token')({
  server: {
    handlers: {
      GET: async () => {
        const token = await readDaemonToken()
        if (!token) return Response.json({ token: null }, { status: 404 })
        return Response.json({ token })
      },
    },
  },
})
