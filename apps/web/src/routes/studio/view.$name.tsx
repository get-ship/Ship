import { createFileRoute } from '@tanstack/react-router'
import { useEffect, useRef, useState } from 'react'
import { MonitorPlay } from 'lucide-react'
import { DAEMON_BASE_URL } from '#/lib/daemon-config'
import { usePlatform } from '#/features/studio/PlatformProvider'
import { installViewBridge } from '#/lib/view-bridge'

export const Route = createFileRoute('/studio/view/$name')({
  component: ExtensionViewPage,
  ssr: false,
})

function buildFrameSrc(name: string, token: string): string {
  const path = `/api/view/${encodeURIComponent(name)}/raw`
  return `${DAEMON_BASE_URL}${path}?token=${encodeURIComponent(token)}`
}

function ExtensionViewPage() {
  const { name } = Route.useParams()
  const { client, daemon, toggleCommandPalette } = usePlatform()
  const frameRef = useRef<HTMLIFrameElement>(null)
  const [status, setStatus] = useState<'loading' | 'ready' | 'error'>('loading')
  const [errorMsg, setErrorMsg] = useState('')

  useEffect(() => {
    return installViewBridge({ onCmdK: () => toggleCommandPalette() })
  }, [toggleCommandPalette])

  if (daemon !== 'connected') {
    return <ViewSkeleton />
  }

  const token = client.getToken()
  if (!token) return <ViewSkeleton />

  const src = buildFrameSrc(name, token)

  return (
    <div className="relative flex-1 min-h-0">
      {status === 'loading' && <ViewSkeleton />}
      {status === 'error' ? (
        <div className="flex items-center justify-center h-full text-muted-foreground">
          <div className="text-center">
            <MonitorPlay className="size-6 mx-auto mb-2 opacity-40" />
            <p className="text-sm font-medium">Failed to load view</p>
            <p className="text-xs mt-1">{errorMsg}</p>
          </div>
        </div>
      ) : (
        <iframe
          ref={frameRef}
          key={name}
          src={src}
          title={name}
          sandbox="allow-scripts allow-same-origin allow-forms"
          className="w-full h-full border-0 block"
          style={{ visibility: status === 'ready' ? 'visible' : 'hidden' }}
          onLoad={() => setStatus('ready')}
          onError={() => {
            setErrorMsg('iframe failed to load')
            setStatus('error')
          }}
        />
      )}
    </div>
  )
}

function ViewSkeleton() {
  return (
    <div className="flex items-center justify-center h-full text-muted-foreground">
      <div className="text-center">
        <MonitorPlay className="size-6 mx-auto mb-2 opacity-40 animate-pulse" />
        <p className="text-xs">Loading view…</p>
      </div>
    </div>
  )
}
