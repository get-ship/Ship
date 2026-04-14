// Merged agent hook: combines pull data with draft overlays.
// This is the primary read path for agent data in the UI.

import { useMemo } from 'react'
import { usePullAgents } from '#/features/studio/hooks/useWorkspaceConfig'
import { useDaemon } from '#/features/studio/hooks/useDaemonData'
import { useAgentDrafts } from './useAgentDrafts'
import { pullAgentToResolved } from './pull-adapter'
import type { ResolvedAgentProfile } from './types'

export interface UseAgentsReturn {
  agents: ResolvedAgentProfile[]
  isLoading: boolean
  isConnected: boolean
  getAgent: (id: string) => ResolvedAgentProfile | undefined
}

export function useAgents(): UseAgentsReturn {
  const { connected } = useDaemon()
  const pullQuery = usePullAgents()
  const { drafts } = useAgentDrafts()

  const pulledAgents = useMemo(() => {
    if (!pullQuery.data?.agents) return []
    return pullQuery.data.agents.map(pullAgentToResolved)
  }, [pullQuery.data])

  const agents = useMemo(() => {
    const draftKeys = Object.keys(drafts)
    if (draftKeys.length === 0) return pulledAgents
    return pulledAgents.map((agent) => {
      const draft = drafts[agent.profile.id]
      if (!draft) return agent
      return mergeAgentWithDraft(agent, draft)
    })
  }, [pulledAgents, drafts])

  const getAgent = useMemo(() => {
    const map = new Map(agents.map((a) => [a.profile.id, a]))
    return (id: string) => map.get(id)
  }, [agents])

  return {
    agents,
    isLoading: pullQuery.isLoading,
    isConnected: connected,
    getAgent,
  }
}

function mergeAgentWithDraft(
  agent: ResolvedAgentProfile,
  draft: Partial<ResolvedAgentProfile>,
): ResolvedAgentProfile {
  const merged = { ...agent, ...draft }
  if (draft.profile) {
    merged.profile = { ...agent.profile, ...draft.profile }
  }
  return merged as ResolvedAgentProfile
}
