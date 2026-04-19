import { type ReactNode } from 'react'
import { Loader2 } from 'lucide-react'
import { usePlatform } from './PlatformProvider'
import { DaemonRequired } from './DaemonRequired'

/**
 * Wraps a daemon-dependent view. Shows DaemonRequired when disconnected,
 * a loading indicator while connecting. Renders children only when connected.
 */
export function DaemonGuard({ children }: { children: ReactNode }) {
  const { daemon } = usePlatform()
  if (daemon === 'disconnected') return <DaemonRequired />
  if (daemon === 'connecting') {
    return (
      <div className="flex flex-col items-center justify-center gap-3 py-16">
        <Loader2 className="size-5 text-muted-foreground animate-spin" />
        <p className="text-sm text-muted-foreground">Connecting to daemon...</p>
      </div>
    )
  }
  return <>{children}</>
}
