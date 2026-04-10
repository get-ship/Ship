import { type ReactNode } from 'react'
import { usePlatform } from './PlatformProvider'
import { DaemonRequired } from './DaemonRequired'

/**
 * Wraps a daemon-dependent view. Shows DaemonRequired when disconnected,
 * nothing while connecting (prevents queries firing without auth).
 * Renders children only when connected.
 */
export function DaemonGuard({ children }: { children: ReactNode }) {
  const { daemon } = usePlatform()
  if (daemon === 'disconnected') return <DaemonRequired />
  if (daemon === 'connecting') return null
  return <>{children}</>
}
