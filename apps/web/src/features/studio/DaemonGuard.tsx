import { type ReactNode } from 'react'
import { usePlatform } from './PlatformProvider'
import { DaemonRequired } from './DaemonRequired'

/**
 * Wraps a daemon-dependent view. Shows DaemonRequired fallback when
 * the daemon is disconnected. Renders children when connected or connecting.
 */
export function DaemonGuard({ children }: { children: ReactNode }) {
  const { daemon } = usePlatform()
  if (daemon === 'disconnected') return <DaemonRequired />
  return <>{children}</>
}
