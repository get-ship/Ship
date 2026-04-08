// Daemon data hooks — replaces the legacy useDaemon.ts.
// All data flows through @ship/sdk's ShipClient (available via useShipClient).

import { useState } from 'react'
import { useShipQuery } from './useShipQuery'
import { useShipClient } from '../ShipClientProvider'
import type { Workspace, JobRecord } from '@ship/ui'

// ── Types ──────────────────────────────────────────────────────────────────────

export interface AgentEntry {
  agent_id: string
  label: string
  capabilities: string[]
  status: string
}

export interface SessionEntry {
  id: string
  workspace_id: string
  workspace_branch: string
  status: string
  started_at: string | null
  ended_at: string | null
  agent_id: string | null
  primary_provider: string | null
  goal: string | null
  summary: string | null
  tool_call_count: number
}

export interface UseDaemonReturn {
  connected: boolean
  workspaces: Workspace[]
  agents: AgentEntry[]
  sessions: SessionEntry[]
  jobs: JobRecord[]
  error: Error | null
}

export interface CreateJobInput {
  slug: string
  agent: string
  branch: string
  spec_path: string
  depends_on: string[] | null
  pipeline?: string
}

export interface CreateWorkspaceInput {
  branch: string
  agent_id: string
  base_branch?: string
}

export interface ViewEntry {
  name: string
}

// ── Helpers ───────────────────────────────────────────────────────────────────

const TEST_BRANCH_PREFIXES = ['feature/evt-', 'feature/rebuild-src-', 'feature/test-', 'test/']

function isTestWorkspace(ws: Workspace): boolean {
  return TEST_BRANCH_PREFIXES.some((p) => ws.branch.startsWith(p))
}

interface MutateOpts<T> {
  onSuccess?: (data: T) => void
  onError?: (err: Error) => void
}

function toError(e: unknown): Error {
  return e instanceof Error ? e : new Error(String(e))
}

// ── Queries ───────────────────────────────────────────────────────────────────

export function useDaemon(): UseDaemonReturn {
  const wsResult = useShipQuery<{ workspaces: Workspace[] }>('/runtime/workspaces')
  const agentsResult = useShipQuery<{ agents: AgentEntry[] }>('/runtime/agents')
  const jobsResult = useShipQuery<{ jobs: JobRecord[] }>('/runtime/jobs')

  const allWorkspaces = wsResult.data?.workspaces ?? []
  const workspaces = allWorkspaces
    .filter((ws) => !isTestWorkspace(ws))
    .sort((a, b) => a.branch.localeCompare(b.branch))

  const activeWsId = allWorkspaces.find((w) => w.status === 'active')?.branch
  const sessionsPath = activeWsId
    ? `/runtime/sessions?workspace_id=${encodeURIComponent(activeWsId)}`
    : null
  const sessionsResult = useShipQuery<{ sessions: SessionEntry[] }>(sessionsPath)

  return {
    connected: !wsResult.error && wsResult.data !== undefined,
    workspaces,
    agents: agentsResult.data?.agents ?? [],
    sessions: sessionsResult.data?.sessions ?? [],
    jobs: (jobsResult.data?.jobs ?? []).sort((a, b) => a.created_at.localeCompare(b.created_at)),
    error: wsResult.error ?? agentsResult.error ?? null,
  }
}

export function useViews() {
  const result = useShipQuery<{ views: ViewEntry[] }>('/runtime/views')
  return { ...result, data: result.data?.views }
}

// ── Mutations ─────────────────────────────────────────────────────────────────

export function useCreateJob() {
  const client = useShipClient()
  const [isPending, setIsPending] = useState(false)
  function mutate(input: CreateJobInput, opts?: MutateOpts<void>) {
    setIsPending(true)
    client.post('/runtime/jobs', input)
      .then(() => { client.invalidatePrefix('/runtime/jobs'); opts?.onSuccess?.() })
      .catch((e: unknown) => opts?.onError?.(toError(e)))
      .finally(() => setIsPending(false))
  }
  return { mutate, isPending }
}

export function useUpdateJob() {
  const client = useShipClient()
  const [isPending, setIsPending] = useState(false)
  function mutate(input: { job_id: string; status: string; error?: string }, opts?: MutateOpts<void>) {
    setIsPending(true)
    client.post('/runtime/jobs/update', input)
      .then(() => { client.invalidatePrefix('/runtime/jobs'); opts?.onSuccess?.() })
      .catch((e: unknown) => opts?.onError?.(toError(e)))
      .finally(() => setIsPending(false))
  }
  return { mutate, isPending }
}

export function useActivateWorkspace() {
  const client = useShipClient()
  const [isPending, setIsPending] = useState(false)
  function mutate(branch: string, opts?: MutateOpts<void>) {
    setIsPending(true)
    client.post(`/workspaces/${encodeURIComponent(branch)}/activate`)
      .then(() => { client.invalidatePrefix('/runtime/workspaces'); opts?.onSuccess?.() })
      .catch((e: unknown) => opts?.onError?.(toError(e)))
      .finally(() => setIsPending(false))
  }
  return { mutate, isPending }
}

export function useCreateWorkspace() {
  const client = useShipClient()
  const [isPending, setIsPending] = useState(false)
  function mutate(input: CreateWorkspaceInput, opts?: MutateOpts<void>) {
    setIsPending(true)
    client.post(`/supervisor/workspaces/${encodeURIComponent(input.branch)}/start`, {
      agent_id: input.agent_id,
      base_branch: input.base_branch ?? 'main',
      open_terminal: false,
    })
      .then(() => { client.invalidatePrefix('/runtime/workspaces'); opts?.onSuccess?.() })
      .catch((e: unknown) => opts?.onError?.(toError(e)))
      .finally(() => setIsPending(false))
  }
  return { mutate, isPending }
}

export function useDeleteWorkspace() {
  const client = useShipClient()
  const [isPending, setIsPending] = useState(false)
  function mutate(branch: string, opts?: MutateOpts<void>) {
    setIsPending(true)
    client.del(`/workspaces/${encodeURIComponent(branch)}`)
      .then(() => { client.invalidatePrefix('/runtime/workspaces'); opts?.onSuccess?.() })
      .catch((e: unknown) => opts?.onError?.(toError(e)))
      .finally(() => setIsPending(false))
  }
  return { mutate, isPending }
}

