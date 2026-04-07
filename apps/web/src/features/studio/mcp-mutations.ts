// MCP mutations — write operations against the shipd daemon.
// Extracted from mcp-queries.ts to keep both files under the 300-line cap.

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { mcpKeys } from '#/lib/query-keys'
import { DAEMON_BASE_URL } from '#/lib/daemon-config'
import { useDaemon } from '#/features/studio/hooks/useDaemon'
import { useStudioShell } from '#/features/studio/StudioShellContext'
import type { TransferBundle } from '@ship/ui'

function useActiveWorkspaceId(): string | null {
  const { viewedWorkspace } = useStudioShell()
  const { workspaces } = useDaemon()
  if (viewedWorkspace) return viewedWorkspace
  return workspaces.find((w) => String(w.status).toLowerCase() === 'active')?.branch ?? null
}

/** Push a transfer bundle to daemon (.ship/). */
export function usePushBundle() {
  const wsId = useActiveWorkspaceId()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (bundle: TransferBundle): Promise<void> => {
      if (!wsId) throw new Error('No active workspace')
      const res = await fetch(
        `${DAEMON_BASE_URL}/api/workspaces/${encodeURIComponent(wsId)}/agents`,
        {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(bundle),
        },
      )
      if (!res.ok) throw new Error(`daemon: push bundle ${res.status}`)
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: mcpKeys.pull() })
    },
  })
}

/** Write a single skill file to disk via daemon. */
export function useSaveSkillFile() {
  const wsId = useActiveWorkspaceId()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (args: { skillId: string; filePath: string; content: string }): Promise<void> => {
      if (!wsId) throw new Error('No active workspace')
      const res = await fetch(
        `${DAEMON_BASE_URL}/api/workspaces/${encodeURIComponent(wsId)}/skills/${encodeURIComponent(args.skillId)}/files/${encodeURIComponent(args.filePath)}`,
        {
          method: 'PUT',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ content: args.content }),
        },
      )
      if (!res.ok) throw new Error(`daemon: save skill file ${res.status}`)
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: mcpKeys.projectSkills() })
    },
  })
}

/** Get merged var values for a skill. */
export function useSkillVars(skillId: string | null) {
  const wsId = useActiveWorkspaceId()

  return useQuery({
    queryKey: mcpKeys.skillVars(skillId ?? ''),
    queryFn: async (): Promise<Record<string, unknown>> => {
      const res = await fetch(
        `${DAEMON_BASE_URL}/api/workspaces/${encodeURIComponent(wsId!)}/skills/${encodeURIComponent(skillId!)}/vars`,
      )
      if (!res.ok) throw new Error(`daemon: skill vars ${res.status}`)
      const body = (await res.json()) as { ok: boolean; data: Record<string, unknown> }
      return body.data
    },
    enabled: wsId != null && skillId != null,
    staleTime: 5_000,
  })
}

/** Set a single skill variable value. */
export function useSetSkillVar() {
  const wsId = useActiveWorkspaceId()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (args: { skillId: string; key: string; valueJson: string }): Promise<void> => {
      if (!wsId) throw new Error('No active workspace')
      const res = await fetch(
        `${DAEMON_BASE_URL}/api/workspaces/${encodeURIComponent(wsId)}/skills/${encodeURIComponent(args.skillId)}/vars/${encodeURIComponent(args.key)}`,
        {
          method: 'PUT',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ value_json: args.valueJson }),
        },
      )
      if (!res.ok) throw new Error(`daemon: set skill var ${res.status}`)
    },
    onSuccess: (_data, args) => {
      void queryClient.invalidateQueries({ queryKey: mcpKeys.skillVars(args.skillId) })
    },
  })
}

/** Delete a single skill file from disk via daemon. */
export function useDeleteSkillFile() {
  const wsId = useActiveWorkspaceId()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (args: { skillId: string; filePath: string }): Promise<void> => {
      if (!wsId) throw new Error('No active workspace')
      const res = await fetch(
        `${DAEMON_BASE_URL}/api/workspaces/${encodeURIComponent(wsId)}/skills/${encodeURIComponent(args.skillId)}/files/${encodeURIComponent(args.filePath)}`,
        { method: 'DELETE' },
      )
      if (!res.ok) throw new Error(`daemon: delete skill file ${res.status}`)
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: mcpKeys.projectSkills() })
    },
  })
}
