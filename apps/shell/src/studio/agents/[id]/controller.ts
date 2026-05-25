import { controller, derived, effect, live, navigate, signal, unwrap } from '@ship/sui-runtime'
import type { LiveSignal, ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { studioRoutes } from '../../routes.gen'
import { HUES, pickHue, pickInitials } from './hue'
import { ops } from '../../../ops'

type Permissions = {
  preset?: string
  tools_allow?: string[]
  tools_ask?: string[]
  tools_deny?: string[]
}

type AgentDetail = {
  id: string
  name: string
  description: string
  letter: string
  hue?: string | null
  icon?: string | null
  version?: string | null
  tags?: string[]
  skills: string[]
  mcp_servers: string[]
  rules: string[]
  inline_rules?: string
  providers: string[]
  default_provider?: string | null
  permissions_tier: string
  permissions?: Permissions
  source: string
}

type WireResp = { agent?: AgentDetail } | null | undefined
type Banner = { ok: true; msg: string } | { ok: false; msg: string } | null


async function readActiveAgent(): Promise<string> {
  try {
    const body = JSON.parse((await ops.settings.state({})).json) as {
      data?: Record<string, unknown>
    }
    const value = body.data?.['shell.agent.active']
    if (typeof value === 'string') return value
    return ''
  } catch {
    return ''
  }
}

export default controller({
  scope: ({ route, ship }: ViewScope<Scope>) => {
    const agentId = derived(() => route.params.id() ?? '')
    const wireCtx = live(() => ship.agents.wire({ id: agentId() }))
    // `wireCtx` emits as `unknown` (emitter gap); narrow to a LiveSignal.
    const wireData = derived(() => (unwrap(wireCtx as LiveSignal<WireResp>) ?? null) as WireResp)

    const agent = derived<AgentDetail | null>(() => {
      const d = wireData()
      return (d && d.agent) ? d.agent : null
    })

    const notFound = derived(() => {
      const ctx = unwrap(wireCtx as LiveSignal<WireResp>)
      if (typeof ctx === 'undefined') return false
      const d = ctx as WireResp
      return !(d && d.agent && typeof d.agent.id === 'string')
    })

    // Hue: prefer the value the author saved on the agent record so
    // the keystone tile matches the tile that was picked during
    // create. Fall back to a deterministic hash so older agents that
    // pre-date the hue field still get a stable colour.
    const hue = derived(() => {
      const a = agent()
      const saved = a && typeof a.hue === 'string' ? a.hue.trim() : ''
      if (saved.length > 0 && (HUES as readonly string[]).includes(saved)) return saved
      return pickHue(a?.id ?? agentId())
    })

    const initials = derived(() => {
      const a = agent()
      const letter = a && typeof a.letter === 'string' ? a.letter.trim() : ''
      if (letter.length > 0) return letter.slice(0, 2).toUpperCase()
      return pickInitials(a?.id ?? agentId(), a?.name)
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
    const inlineRulesText = derived(() => {
      const a = agent()
      return (a && typeof a.inline_rules === 'string') ? a.inline_rules : ''
    })
    const hasInlineRules = derived(() => inlineRulesText().trim().length > 0)
    const rulesTotal = derived(() => rulesList().length + (hasInlineRules() ? 1 : 0))

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

    const permLists = derived(() => {
      const a = agent()
      const p = a?.permissions ?? {}
      return {
        allow: Array.isArray(p.tools_allow) ? p.tools_allow : [],
        ask: Array.isArray(p.tools_ask) ? p.tools_ask : [],
        deny: Array.isArray(p.tools_deny) ? p.tools_deny : [],
      }
    })
    const permTotalCount = derived(() => {
      const p = permLists()
      return p.allow.length + p.ask.length + p.deny.length
    })

    const sourcePath = derived(() => {
      const a = agent()
      const id = (a && a.id) || agentId() || 'agent'
      const src = a && typeof a.source === 'string' ? a.source : ''
      const root = src.includes('USER') ? '~/.ship/library/agents/' : '.ship/agents/'
      return `${root}${id}.jsonc`
    })

    // Meta separators are rendered conditionally; this captures the
    // fields the detail header shows beside the id chip so the
    // template can emit a separator between each present value.
    const metaBits = derived(() => {
      const a = agent()
      if (!a) return []
      const bits: string[] = []
      if (a.source) bits.push(a.source)
      if (a.version) bits.push(`v${a.version}`)
      return bits
    })

    // Active-agent — workspace-scoped via the `shell.agent.active` setting.
    const activeAgentId = signal('')
    const activating = signal(false)
    const banner = signal<Banner>(null)
    void readActiveAgent().then((id) => activeAgentId.set(id))

    const isActive = derived(() => {
      const id = agentId()
      return id !== '' && activeAgentId() === id
    })

    // Clear stale banners when the loaded agent changes.
    effect(() => {
      void agent()
      banner.set(null)
    })

    return {
      agentId, wireCtx, wireData, agent, notFound,
      hue, initials, description, metaBits,
      skillsList, mcpList, rulesList, hasInlineRules, inlineRulesText, rulesTotal,
      providerList, stats, permPreset, permLists, permTotalCount, sourcePath,
      activeAgentId, activating, isActive, banner,
    }
  },
  on: {
    goWire: (s) => navigate(studioRoutes.agentsIdWire({ id: s.agentId })),
    goSettings: (s) => navigate(studioRoutes.agentsIdSettings({ id: s.agentId })),
    // Segmented tabs: the control swallows per-option clicks, so navigate off
    // its sui-change event instead. overview → detail root; else → sub-route.
    goTab: (s, value: string) => {
      const id = s.agentId
      navigate(value === 'overview' ? studioRoutes.agentsId({ id }) : `/studio/agents/${id}/${value}`)
    },
    goBack: () => navigate('/studio/agents'),
    goSkill: (_s, id: string) => navigate(studioRoutes.skillsId({ id })),

    activate: async (s) => {
      if (s.activating) return
      s.activating = true
      s.banner = null
      try {
        await ops.settings.set({
          key: 'shell.agent.active',
          valueJson: JSON.stringify(s.agentId),
        })
        s.activeAgentId = s.agentId
        s.banner = { ok: true, msg: 'Agent activated.' }
      } catch (err) {
        s.banner = { ok: false, msg: (err as Error).message ?? 'Activate failed.' }
      } finally {
        s.activating = false
      }
    },
    deactivate: async (s) => {
      if (s.activating) return
      s.activating = true
      s.banner = null
      try {
        await ops.settings.set({ key: 'shell.agent.active', valueJson: JSON.stringify('') })
        s.activeAgentId = ''
        s.banner = { ok: true, msg: 'Agent cleared.' }
      } catch (err) {
        s.banner = { ok: false, msg: (err as Error).message ?? 'Deactivate failed.' }
      } finally {
        s.activating = false
      }
    },
  },
})
