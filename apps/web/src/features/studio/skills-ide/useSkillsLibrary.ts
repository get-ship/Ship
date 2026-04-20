// Skills library: aggregates skills from MCP pull data into a deduplicated view.
// Skills come from pulled agents (PullAgent.skills). Each skill tracks which
// agents reference it and its origin (project vs library).

import { useMemo } from 'react'
import { usePullAgents, useProjectSkills } from '#/features/studio/hooks/useWorkspaceConfig'
import { useDaemon } from '#/features/studio/hooks/useDaemonData'
import type { Skill, PullSkill, JsonValue } from '@ship/set'

export interface LibrarySkill extends Skill {
  /** 'project' = from .ship/, 'library' = from ~/.ship/ */
  origin: 'project' | 'library'
  /** Agent IDs that reference this skill */
  usedBy: string[]
  /** Canonical storage key from stable-id frontmatter */
  stableId: string | null
  /** Tags from frontmatter */
  tags: string[]
  /** Authors from frontmatter */
  authors: string[]
  /** Raw vars.json schema, null if no vars */
  varsSchema: JsonValue | null
  /** All files in the skill directory */
  files: string[]
  /** Reference doc content keyed by relative path */
  referenceDocs: Record<string, string>
  /** Raw evals.json content, null if no evals */
  evals: JsonValue | null
}

export interface UseSkillsLibraryReturn {
  skills: LibrarySkill[]
  isLoading: boolean
  isConnected: boolean
}

/** Convert a PullSkill to a LibrarySkill. */
function pullToLibrarySkill(
  ps: PullSkill,
  origin: 'project' | 'library',
  usedBy: string[],
): LibrarySkill {
  const source = (ps.source === 'custom' || ps.source === 'builtin' ||
           ps.source === 'ai-generated' || ps.source === 'community' ||
           ps.source === 'imported')
    ? ps.source
    : 'custom' as const
  return {
    id: ps.id,
    name: ps.name,
    description: ps.description ?? null,
    content: ps.content,
    source,
    artifacts: ps.artifacts ?? [],
    vars: {},
    origin,
    usedBy,
    stableId: ps.stable_id ?? null,
    tags: ps.tags ?? [],
    authors: ps.authors ?? [],
    varsSchema: ps.vars_schema ?? null,
    files: ps.files ?? [],
    referenceDocs: (ps.reference_docs ?? {}) as Record<string, string>,
    evals: ps.evals ?? null,
  }
}

/**
 * Aggregate and deduplicate skills from all pulled agents.
 * Returns a stable array of LibrarySkill with usage and origin metadata.
 */
export function aggregateSkills(
  agents: { id: string; skills: PullSkill[]; source?: string }[],
): LibrarySkill[] {
  const map = new Map<string, LibrarySkill>()

  for (const agent of agents) {
    const agentOrigin = agent.source === 'library' ? 'library' : 'project'

    for (const ps of agent.skills) {
      const existing = map.get(ps.id)
      if (existing) {
        if (!existing.usedBy.includes(agent.id)) {
          existing.usedBy.push(agent.id)
        }
      } else {
        map.set(ps.id, pullToLibrarySkill(
          ps,
          agentOrigin as 'project' | 'library',
          [agent.id],
        ))
      }
    }
  }

  return Array.from(map.values())
}

/** Merge project skills into an agent-derived skill list. Deduplicates by ID. */
export function mergeProjectSkills(
  agentSkills: LibrarySkill[],
  projectPullSkills: PullSkill[],
): LibrarySkill[] {
  const map = new Map<string, LibrarySkill>()
  for (const s of agentSkills) map.set(s.id, s)
  for (const ps of projectPullSkills) {
    if (!map.has(ps.id)) {
      map.set(ps.id, pullToLibrarySkill(ps, 'project', []))
    }
  }
  return Array.from(map.values())
}

export function useSkillsLibrary(): UseSkillsLibraryReturn {
  const { connected: isConnected } = useDaemon()
  const pullQuery = usePullAgents()
  const projectQuery = useProjectSkills()

  const pulledSkills = useMemo(() => {
    if (!pullQuery.data?.agents) return []
    const agentInputs = pullQuery.data.agents.map((a) => ({
      id: a.profile.id,
      skills: a.skills,
      source: a.source,
    }))
    return aggregateSkills(agentInputs)
  }, [pullQuery.data])

  const agentUsageMap = useMemo(() => {
    const map = new Map<string, string[]>()
    if (!pullQuery.data?.agents) return map
    for (const agent of pullQuery.data.agents) {
      for (const skill of agent.skills) {
        const existing = map.get(skill.id)
        if (existing) {
          if (!existing.includes(agent.profile.id)) existing.push(agent.profile.id)
        } else {
          map.set(skill.id, [agent.profile.id])
        }
      }
    }
    return map
  }, [pullQuery.data])

  // Primary source: list_project_skills. This is the canonical skill list.
  // Agent data enriches skills with usedBy metadata but does not define the list.
  // Fall back to agent-derived skills only while project query loads.
  const skills = useMemo(() => {
    const projectSkills = projectQuery.data ?? []
    if (projectSkills.length > 0) {
      return projectSkills.map((ps) =>
        pullToLibrarySkill(ps, 'project', agentUsageMap.get(ps.id) ?? []),
      )
    }
    return pulledSkills
  }, [projectQuery.data, agentUsageMap, pulledSkills])

  return {
    skills,
    isLoading: pullQuery.isLoading || projectQuery.isLoading,
    isConnected,
  }
}
