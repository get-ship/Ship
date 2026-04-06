// MCP queries — fetch agent and skill config from the shipd daemon.
// Mutation stubs remain until daemon write endpoints are available.

import { useQuery, useMutation } from '@tanstack/react-query'
import { mcpKeys } from '#/lib/query-keys'
import { DAEMON_BASE_URL } from '#/lib/daemon-config'
import { useDaemon } from '#/features/studio/hooks/useDaemon'
import type {
  PullResponse,
  PullSkill,
  ListAgentsResponse,
  TransferBundle,
} from '@ship/ui'

// ── Helpers ────────────────────────────────────────────────────────────

function useActiveWorkspaceId(): string | null {
  const { workspaces } = useDaemon()
  return workspaces.find((w) => String(w.status).toLowerCase() === 'active')?.branch ?? null
}

// ── Queries ─────────────────────────────────────────────────────────────

/** Fetch local agent IDs from daemon (.ship/agents/). */
export function useLocalAgentIds() {
  const wsId = useActiveWorkspaceId()

  return useQuery({
    queryKey: mcpKeys.agentList(),
    queryFn: async (): Promise<ListAgentsResponse> => {
      const res = await fetch(
        `${DAEMON_BASE_URL}/api/workspaces/${encodeURIComponent(wsId!)}/agents`,
      )
      if (!res.ok) throw new Error(`daemon: agents ${res.status}`)
      const body = (await res.json()) as {
        ok: boolean
        data: { agents: Array<{ id: string }> }
      }
      return { agents: body.data.agents.map((a) => a.id) }
    },
    enabled: wsId != null,
    staleTime: 5_000,
  })
}

/** Pull all resolved agents from daemon (.ship/). */
export function usePullAgents() {
  const wsId = useActiveWorkspaceId()

  return useQuery({
    queryKey: mcpKeys.pull(),
    queryFn: async (): Promise<PullResponse> => {
      // Fetch agents + skills in parallel from daemon
      const [agentsRes, skillsRes] = await Promise.all([
        fetch(`${DAEMON_BASE_URL}/api/workspaces/${encodeURIComponent(wsId!)}/agents`),
        fetch(`${DAEMON_BASE_URL}/api/workspaces/${encodeURIComponent(wsId!)}/skills`),
      ])
      if (!agentsRes.ok) throw new Error(`daemon: agents ${agentsRes.status}`)
      if (!skillsRes.ok) throw new Error(`daemon: skills ${skillsRes.status}`)

      const agentsBody = (await agentsRes.json()) as {
        ok: boolean
        data: {
          agents: Array<{
            id: string
            name: string
            description?: string
            skills?: { refs: string[] }
            providers?: string[]
          }>
        }
      }
      const skillsBody = (await skillsRes.json()) as {
        ok: boolean
        data: {
          skills: Array<{
            id: string
            name: string
            description?: string | null
            files: string[]
          }>
        }
      }

      // Build skill lookup for resolving refs
      const skillMap = new Map(
        skillsBody.data.skills.map((s) => [s.id, s]),
      )

      const agents = agentsBody.data.agents.map((a) => {
        const skillRefs = a.skills?.refs ?? []
        const resolvedSkills = skillRefs
          .map((ref) => skillMap.get(ref))
          .filter(Boolean)
          .map((s) => ({
            id: s!.id,
            name: s!.name,
            description: s!.description ?? null,
            content: '',
            source: 'project',
            artifacts: [],
          }))

        return {
          profile: {
            id: a.id,
            name: a.name,
            description: a.description ?? '',
            providers: a.providers ?? [],
            version: '0.0.0',
          },
          skills: resolvedSkills,
          mcpServers: [],
          rules: [],
          hooks: [],
        }
      })

      const allSkills = skillsBody.data.skills.map((s) => ({
        id: s.id,
        name: s.name,
        description: s.description ?? null,
        content: '',
        source: 'project' as const,
        artifacts: [],
      }))

      return { agents, skills: allSkills } as unknown as PullResponse
    },
    enabled: wsId != null,
    staleTime: 5_000,
  })
}

/** Fetch all project skills from daemon (.ship/skills/). */
export function useProjectSkills() {
  const wsId = useActiveWorkspaceId()

  return useQuery({
    queryKey: mcpKeys.projectSkills(),
    queryFn: async (): Promise<PullSkill[]> => {
      const res = await fetch(
        `${DAEMON_BASE_URL}/api/workspaces/${encodeURIComponent(wsId!)}/skills`,
      )
      if (!res.ok) throw new Error(`daemon: skills ${res.status}`)
      const body = (await res.json()) as {
        ok: boolean
        data: { skills: PullSkill[] }
      }
      return body.data.skills
    },
    enabled: wsId != null,
    staleTime: 5_000,
  })
}

// ── Mutations (no-op stubs) ────────────────────────────────────────────

/** Push a transfer bundle to CLI (.ship/). */
export function usePushBundle() {
  return useMutation({
    mutationFn: async (_bundle: TransferBundle): Promise<string> => {
      console.warn('usePushBundle: stubbed — daemon write endpoint not yet available')
      return ''
    },
  })
}

/** Write a single skill file to disk via CLI. */
export function useSaveSkillFile() {
  return useMutation({
    mutationFn: async (_args: { skillId: string; filePath: string; content: string }): Promise<string> => {
      console.warn('useSaveSkillFile: stubbed — daemon write endpoint not yet available')
      return ''
    },
  })
}

/** Get merged var values for a skill. */
export function useSkillVars(skillId: string | null) {
  return useQuery({
    queryKey: mcpKeys.skillVars(skillId ?? ''),
    queryFn: async () => {
      console.warn('useSkillVars: stubbed — daemon endpoint not yet available')
      return {} as Record<string, unknown>
    },
    enabled: skillId != null,
    staleTime: 5_000,
  })
}

/** Set a single skill variable value. */
export function useSetSkillVar() {
  return useMutation({
    mutationFn: async (_args: { skillId: string; key: string; valueJson: string }) => {
      console.warn('useSetSkillVar: stubbed — daemon write endpoint not yet available')
      return ''
    },
  })
}

/** Delete a single skill file from disk via CLI. */
export function useDeleteSkillFile() {
  return useMutation({
    mutationFn: async (_args: { skillId: string; filePath: string }) => {
      console.warn('useDeleteSkillFile: stubbed — daemon write endpoint not yet available')
      return ''
    },
  })
}
