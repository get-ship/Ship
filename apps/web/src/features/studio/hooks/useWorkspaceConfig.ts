// Workspace config hooks — replaces mcp-queries.ts + mcp-mutations.ts.
// All data flows through @ship/sdk's ShipClient (available via usePlatform().client).

import { useState, useMemo } from 'react'
import { useShipQuery } from './useShipQuery'
import { usePlatform } from '../PlatformProvider'
import type {
  PullResponse,
  PullSkill,
  ListAgentsResponse,
  TransferBundle,
} from '@ship/ui'

// ── Helpers ──────────────────────────────────────────────────────────────────

export function useActiveWorkspaceId(): string {
  const { workspace } = usePlatform()
  return workspace?.branch ?? ''
}

// ── Internal response types ───────────────────────────────────────────────────

interface AgentsBody {
  agents: Array<{
    id: string
    name: string
    description?: string
    skills?: { refs: string[] }
    providers?: string[]
    mcp_servers?: Array<{ name: string; command: string; url?: string | null }>
    rules?: Array<{ file_name: string; content: string }>
    hooks?: unknown[]
    model?: string | null
    env?: Record<string, string> | null
    available_models?: string[]
    agent_limits?: Record<string, unknown> | null
    provider_settings?: Record<string, Record<string, unknown>>
  }>
}

interface SkillsBody {
  skills: Array<{
    id: string
    name: string
    stable_id?: string | null
    description?: string | null
    content?: string
    files: string[]
    artifacts?: string[]
    tags?: string[]
    authors?: string[]
    vars_schema?: unknown
    reference_docs?: Record<string, string>
    evals?: unknown
  }>
}

interface MutateOpts<T> {
  onSuccess?: (data: T) => void
  onError?: (err: Error) => void
}

function toError(e: unknown): Error {
  return e instanceof Error ? e : new Error(String(e))
}

// ── Queries ───────────────────────────────────────────────────────────────────

/** Fetch local agent IDs from daemon (.ship/agents/). */
export function useLocalAgentIds() {
  const wsId = useActiveWorkspaceId()
  const path = wsId ? `/workspaces/${encodeURIComponent(wsId)}/agents` : null
  const result = useShipQuery<AgentsBody>(path)
  return {
    ...result,
    data: result.data ? ({ agents: result.data.agents.map((a) => a.id) } as ListAgentsResponse) : undefined,
  }
}

/** Pull all resolved agents + skills from daemon (.ship/). */
export function usePullAgents() {
  const wsId = useActiveWorkspaceId()
  const agentsPath = wsId ? `/workspaces/${encodeURIComponent(wsId)}/agents` : null
  const skillsPath = wsId ? `/workspaces/${encodeURIComponent(wsId)}/skills` : null

  const agentsResult = useShipQuery<AgentsBody>(agentsPath)
  const skillsResult = useShipQuery<SkillsBody>(skillsPath)

  const data = useMemo((): PullResponse | undefined => {
    if (!agentsResult.data || !skillsResult.data) return undefined

    const skillMap = new Map(skillsResult.data.skills.map((s) => [s.id, s]))

    const agents = agentsResult.data.agents.map((a) => {
      const skillRefs = a.skills?.refs ?? []
      const resolvedSkills = skillRefs
        .map((ref) => skillMap.get(ref))
        .filter(Boolean)
        .map((s) => ({
          id: s!.id,
          name: s!.name,
          description: s!.description ?? null,
          content: '',
          source: 'project' as const,
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
        mcpServers: (a.mcp_servers ?? []).map((s) => ({
          name: s.name,
          command: s.command,
          url: s.url ?? null,
        })),
        rules: a.rules ?? [],
        hooks: a.hooks ?? [],
        model: a.model ?? null,
        env: a.env ?? null,
        available_models: a.available_models ?? [],
        agent_limits: a.agent_limits ?? null,
        provider_settings: a.provider_settings ?? {},
      }
    })

    const allSkills = skillsResult.data.skills.map((s) => ({
      id: s.id,
      name: s.name,
      description: s.description ?? null,
      content: '',
      source: 'project' as const,
      artifacts: [],
    }))

    return { agents, skills: allSkills } as unknown as PullResponse
  }, [agentsResult.data, skillsResult.data])

  return {
    data,
    isLoading: agentsResult.isLoading || skillsResult.isLoading,
    error: agentsResult.error ?? skillsResult.error,
    isSuccess: agentsResult.isSuccess && skillsResult.isSuccess,
    refetch: agentsResult.refetch,
  }
}

/** Fetch all project skills from daemon (.ship/skills/). */
export function useProjectSkills() {
  const wsId = useActiveWorkspaceId()
  const path = wsId ? `/workspaces/${encodeURIComponent(wsId)}/skills` : null
  const result = useShipQuery<SkillsBody>(path)
  return {
    ...result,
    data: result.data?.skills.map((s) => ({
      id: s.id,
      name: s.name,
      stable_id: s.stable_id ?? null,
      description: s.description ?? null,
      content: s.content ?? '',
      source: 'custom' as const,
      artifacts: s.artifacts ?? [],
      files: s.files ?? [],
      tags: s.tags ?? [],
      authors: s.authors ?? [],
      vars_schema: s.vars_schema ?? null,
      reference_docs: s.reference_docs ?? {},
      evals: s.evals ?? null,
    })) as PullSkill[] | undefined,
  }
}

/** Get merged var values for a skill. */
export function useSkillVars(skillId: string | null) {
  const wsId = useActiveWorkspaceId()
  const path =
    wsId && skillId
      ? `/workspaces/${encodeURIComponent(wsId)}/skills/${encodeURIComponent(skillId)}/vars`
      : null
  return useShipQuery<Record<string, unknown>>(path)
}

// ── Mutations ─────────────────────────────────────────────────────────────────

/** Push a transfer bundle to daemon (.ship/). */
export function usePushBundle() {
  const { client } = usePlatform()
  const wsId = useActiveWorkspaceId()
  const [isPending, setIsPending] = useState(false)
  function mutate(bundle: TransferBundle, opts?: MutateOpts<void>) {
    if (!wsId) { opts?.onError?.(new Error('No active workspace')); return }
    setIsPending(true)
    client.post(`/workspaces/${encodeURIComponent(wsId)}/agents`, bundle)
      .then(() => { client.invalidatePrefix(`/workspaces/${wsId}`); opts?.onSuccess?.() })
      .catch((e: unknown) => opts?.onError?.(toError(e)))
      .finally(() => setIsPending(false))
  }
  return { mutate, isPending }
}

/** Write a single skill file to disk via daemon. */
export function useSaveSkillFile() {
  const { client } = usePlatform()
  const wsId = useActiveWorkspaceId()
  const [isPending, setIsPending] = useState(false)
  function mutate(args: { skillId: string; filePath: string; content: string }, opts?: MutateOpts<void>) {
    if (!wsId) { opts?.onError?.(new Error('No active workspace')); return }
    setIsPending(true)
    client.put(
      `/workspaces/${encodeURIComponent(wsId)}/skills/${encodeURIComponent(args.skillId)}/files/${encodeURIComponent(args.filePath)}`,
      { content: args.content },
    )
      .then(() => { client.invalidatePrefix(`/workspaces/${wsId}/skills`); opts?.onSuccess?.() })
      .catch((e: unknown) => opts?.onError?.(toError(e)))
      .finally(() => setIsPending(false))
  }
  return { mutate, isPending }
}

/** Set a single skill variable value. */
export function useSetSkillVar() {
  const { client } = usePlatform()
  const wsId = useActiveWorkspaceId()
  const [isPending, setIsPending] = useState(false)
  function mutate(args: { skillId: string; key: string; valueJson: string }, opts?: MutateOpts<void>) {
    if (!wsId) { opts?.onError?.(new Error('No active workspace')); return }
    setIsPending(true)
    client.put(
      `/workspaces/${encodeURIComponent(wsId)}/skills/${encodeURIComponent(args.skillId)}/vars/${encodeURIComponent(args.key)}`,
      { value_json: args.valueJson },
    )
      .then(() => {
        client.invalidatePrefix(`/workspaces/${wsId}/skills/${args.skillId}/vars`)
        opts?.onSuccess?.()
      })
      .catch((e: unknown) => opts?.onError?.(toError(e)))
      .finally(() => setIsPending(false))
  }
  return { mutate, isPending }
}

/** Delete a single skill file from disk via daemon. */
export function useDeleteSkillFile() {
  const { client } = usePlatform()
  const wsId = useActiveWorkspaceId()
  const [isPending, setIsPending] = useState(false)
  function mutate(args: { skillId: string; filePath: string }, opts?: MutateOpts<void>) {
    if (!wsId) { opts?.onError?.(new Error('No active workspace')); return }
    setIsPending(true)
    client.del(
      `/workspaces/${encodeURIComponent(wsId)}/skills/${encodeURIComponent(args.skillId)}/files/${encodeURIComponent(args.filePath)}`,
    )
      .then(() => { client.invalidatePrefix(`/workspaces/${wsId}/skills`); opts?.onSuccess?.() })
      .catch((e: unknown) => opts?.onError?.(toError(e)))
      .finally(() => setIsPending(false))
  }
  return { mutate, isPending }
}
