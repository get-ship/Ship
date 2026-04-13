import { createFileRoute } from '@tanstack/react-router'
import { MonitorPlay } from 'lucide-react'
import { DAEMON_BASE_URL } from '#/lib/daemon-config'
import { usePlatform } from '#/features/studio/PlatformProvider'

export const Route = createFileRoute('/studio/view/$name')({
  component: ExtensionViewPage,
  ssr: false,
})

function ExtensionViewPage() {
  const { name } = Route.useParams()
  const { client, daemon } = usePlatform()

  if (daemon !== 'connected') {
    return (
      <div className="flex items-center justify-center h-full text-muted-foreground">
        <div className="text-center">
          <MonitorPlay className="size-6 mx-auto mb-2 opacity-40" />
          <p className="text-sm font-medium">Waiting for daemon…</p>
        </div>
      </div>
    )
  }

  const token = client.getToken()
  if (!token) return null

  const src = `${DAEMON_BASE_URL}/api/runtime/view/${encodeURIComponent(name)}/raw?token=${encodeURIComponent(token)}`

  return (
    <iframe
      src={src}
      className="flex-1 min-h-0 w-full border-0"
      title={`Ship view: ${name}`}
      sandbox="allow-scripts allow-same-origin allow-forms allow-popups"
    />
  )
}
