import { createFileRoute } from '@tanstack/react-router'
import { useQuery } from '@tanstack/react-query'
import { MonitorPlay } from 'lucide-react'
import { DAEMON_BASE_URL } from '#/lib/daemon-config'
import { ViewHost } from '#/features/studio/views/ViewHost'
import { useDaemon } from '#/features/studio/hooks/useDaemonData'

export const Route = createFileRoute('/studio/view/$name')({
  component: ExtensionViewPage,
  ssr: false,
})

function ExtensionViewPage() {
  const { name } = Route.useParams()
  const { workspaces } = useDaemon()
  const activeWs = workspaces.find((w) => String(w.status).toLowerCase() === 'active')

  const { data: html, isLoading, error } = useQuery({
    queryKey: ['view', name],
    queryFn: async () => {
      const res = await fetch(`${DAEMON_BASE_URL}/api/runtime/view/${encodeURIComponent(name)}`)
      if (!res.ok) throw new Error(`Failed to load view: ${res.status}`)
      const body = await res.json() as { ok: boolean; data: { html: string } }
      return body.data.html
    },
    staleTime: 60_000,
  })

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full text-muted-foreground">
        <div className="text-center">
          <MonitorPlay className="size-6 mx-auto mb-2 opacity-40 animate-pulse" />
          <p className="text-xs">Loading {name}...</p>
        </div>
      </div>
    )
  }

  if (error || !html) {
    return (
      <div className="flex items-center justify-center h-full text-muted-foreground">
        <div className="text-center">
          <MonitorPlay className="size-6 mx-auto mb-2 opacity-40" />
          <p className="text-sm font-medium">Failed to load view</p>
          <p className="text-xs mt-1">{error?.message ?? `View "${name}" not found`}</p>
        </div>
      </div>
    )
  }

  return (
    <div className="flex-1 flex flex-col min-h-0 overflow-hidden">
      <ViewHost html={html} workspaceId={activeWs?.branch} />
    </div>
  )
}
