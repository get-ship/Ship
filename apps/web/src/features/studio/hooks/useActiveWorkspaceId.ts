import { useDaemon } from '#/features/studio/hooks/useDaemonData'
import { useStudioShell } from '#/features/studio/StudioShellContext'

/**
 * Resolve the active workspace branch for data fetching.
 *
 * Uses viewedWorkspace from context if it still exists in the workspace list.
 * Falls back to the most recently activated workspace, then empty string.
 */
export function useActiveWorkspaceId(): string {
  const { viewedWorkspace } = useStudioShell()
  const { workspaces } = useDaemon()

  if (viewedWorkspace && workspaces.some((w) => w.branch === viewedWorkspace)) {
    return viewedWorkspace
  }

  const active = workspaces
    .filter((w) => String(w.status).toLowerCase() === 'active')
    .sort((a, b) => (b.last_activated_at ?? '').localeCompare(a.last_activated_at ?? ''))
  return active[0]?.branch ?? ''
}
