import { controller, derived, live, unwrap } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

type AgentDetail = {
  id: string
  name: string
  description: string
  letter: string
  skills: string[]
  mcp_servers: string[]
  rules: string[]
  providers: string[]
  default_provider?: string | null
  permissions_tier: string
  permissions?: { preset?: string }
  source: string
}

type WireResp = { agent?: AgentDetail } | null | undefined

export default controller<Scope>({
  scope: ({ route, ship }) => {
    const agentId = derived(() => route.params.id() ?? '')
    const wireCtx = live(() => ship.agents.wire({ id: agentId() }))
    const wireData = derived(() => (unwrap(wireCtx as any) ?? null) as WireResp)

    const agent = derived<AgentDetail | null>(() => {
      const d = wireData()
      return (d && d.agent) ? d.agent : null
    })

    // True once the live() resolver has produced a value AND that value
    // lacks an agent. Distinguishes "still loading" (unwrap returns
    // undefined) from "loaded, not found" (unwrap returns null because
    // the daemon emits {value: null} for missing ids rather than an
    // error event). Drives the empty-state UI for unknown ids.
    const notFound = derived(() => {
      const ctx = unwrap(wireCtx as any)
      if (typeof ctx === 'undefined') return false
      const d = ctx as WireResp
      return !(d && d.agent && typeof d.agent.id === 'string')
    })

    const description = derived(() => {
      const a = agent()
      const d = a && typeof a.description === 'string' ? a.description.trim() : ''
      return d.length > 0 ? d : 'No description provided.'
    })

    const skillsList = derived<string[]>(() => {
      const a = agent()
      return a && Array.isArray(a.skills) ? a.skills : []
    })
    const mcpList = derived<string[]>(() => {
      const a = agent()
      return a && Array.isArray(a.mcp_servers) ? a.mcp_servers : []
    })
    const rulesList = derived<string[]>(() => {
      const a = agent()
      return a && Array.isArray(a.rules) ? a.rules : []
    })
    const providerList = derived<{ id: string; label: string; isDefault: boolean }[]>(() => {
      const a = agent()
      if (!a || !Array.isArray(a.providers)) return []
      const def = a.default_provider || (a.providers.length > 0 ? a.providers[0] : '')
      return a.providers.map((id) => ({
        id,
        label: id.length > 0 ? id[0].toUpperCase() + id.slice(1) : id,
        isDefault: id === def,
      }))
    })

    const stats = derived(() => ({
      skills: skillsList().length,
      mcp: mcpList().length,
      rules: rulesList().length,
      providers: providerList().length,
    }))

    const permPreset = derived(() => {
      const a = agent()
      if (!a) return 'none'
      const p = a.permissions && typeof a.permissions.preset === 'string' ? a.permissions.preset : ''
      if (p.length > 0) return p
      return a.permissions_tier || 'none'
    })

    const sourcePath = derived(() => {
      const a = agent()
      const id = (a && a.id) || agentId() || 'agent'
      const src = a && typeof a.source === 'string' ? a.source : ''
      const root = src.includes('USER') ? '~/.ship/library/agents/' : '.ship/agents/'
      return `${root}${id}.jsonc`
    })

    return {
      agentId, wireCtx, wireData, agent, notFound, description,
      skillsList, mcpList, rulesList, providerList,
      stats, permPreset, sourcePath,
    }
  },
  on: {
    goWire: ({ navigate, agentId }) => navigate(`/agents/${agentId}/wire`),
    goSettings: ({ navigate, agentId }) => navigate(`/agents/${agentId}/settings`),
    goBack: ({ navigate }) => navigate('/agents'),
    goSkill: ({ navigate }, id: string) => navigate(`/skills/${id}`),
    openSource: ({ navigate, agentId }) => {
      // Settings is the editor surface for the agent jsonc. The
      // segmented tab also routes here; this button is the explicit
      // "edit raw JSONC" affordance from the overview header.
      navigate(`/agents/${agentId}/settings`)
    },
  },
})
