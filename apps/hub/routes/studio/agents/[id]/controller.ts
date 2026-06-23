// Agent detail (overview tab). Reads the unary `agents.*` Connect surface
// through the ambient `ops` client (host-owned, injected as `clients.ops` by
// entry.gen.ts): `agents.Wire` carries the view-shaped detail (skills, mcp,
// rules, permissions, tags, source, version) and `agents.Get` carries the raw
// agent doc, the source of the two fields the wire projection does NOT expose
// — `providers` and `hue`. `live(() => ops.agents.x(…))` wraps each unary read
// as a LiveSignal so the template's `{#async}` block drives loading/error/ok.
//
// Seams: `navigate` comes from `@ship/deck/router` (the hub's bundled runtime
// does not re-export it); `readSetting`/`writeSetting` live in the sibling
// `settings.ts` shim; `routes` resolves the sibling wire/settings sub-routes.
import { controller, derived, effect, live, signal, unwrap } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { routes } from '../../../../routes.gen'
import { navigate } from '@ship/deck/router'
import { HUES, pickHue, pickInitials } from './hue'
import { readSetting, writeSetting } from './settings'
import type { AgentDetail, ShipConnectClient } from '@ship/sdk/connect'

// The raw agent-doc body (parsed from `AgentGetResponse.docJson`). Only the
// fields the wire projection drops are read here — `agent.providers` and
// `agent.hue` — so the shape stays intentionally loose.
type DocAgent = {
  providers?: string[]
  default_provider?: string | null
  hue?: string | null
} | null

type Banner = { ok: true; msg: string } | { ok: false; msg: string } | null


async function readActiveAgent(ops: ShipConnectClient): Promise<string> {
  const value = await readSetting<unknown>(ops, 'shell.agent.active')
  return typeof value === 'string' ? value : ''
}

export default controller({
  scope: (ctx: ViewScope<Scope>) => {
    // AMBIENT client (host-owned): the entry.gen.ts bootstrap constructs
    // `ops` from `src/lib/client.ts` and injects it as `clients.ops`.
    const { route } = ctx
    const ops = ctx.clients.ops
    const agentId = derived(() => route.params.id() ?? '')
    const wireCtx = live(() => ops.agents.wire({ id: agentId() }))
    // Raw doc — sole source of `providers`/`hue` (absent from the wire
    // projection). Empty scope walks Project → User.
    const docCtx = live(() => ops.agents.get({ id: agentId(), scope: '' }))
    const wireData = derived(() => unwrap(wireCtx) ?? null)

    const agent = derived<AgentDetail | null>(() => wireData()?.agent ?? null)

    const docAgent = derived<DocAgent>(() => {
      const r = unwrap(docCtx)
      if (!r) return null
      try {
        const doc = JSON.parse(r.docJson) as { agent?: DocAgent }
        return doc.agent ?? null
      } catch {
        return null
      }
    })

    const notFound = derived(() => {
      const st = wireCtx()
      if (st.kind !== 'ok') return false
      const a = st.value?.agent
      return !(a && typeof a.id === 'string')
    })

    // Hue: prefer the value the author saved on the agent doc so the
    // keystone tile matches the tile picked during create. Fall back to a
    // deterministic hash so agents that declare no hue still get a stable
    // colour.
    const hue = derived(() => {
      const saved = typeof docAgent()?.hue === 'string' ? (docAgent()!.hue as string).trim() : ''
      if (saved.length > 0 && (HUES as readonly string[]).includes(saved)) return saved
      return pickHue(agent()?.id ?? agentId())
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
      return a && Array.isArray(a.mcpServers) ? a.mcpServers : []
    })
    const rulesList = derived<string[]>(() => {
      const a = agent()
      return a && Array.isArray(a.rules) ? a.rules : []
    })
    const inlineRulesText = derived(() => {
      const a = agent()
      return (a && typeof a.inlineRules === 'string') ? a.inlineRules : ''
    })
    const hasInlineRules = derived(() => inlineRulesText().trim().length > 0)
    const rulesTotal = derived(() => rulesList().length + (hasInlineRules() ? 1 : 0))

    // Providers + default come from the raw doc (`agent.providers`), not the
    // wire projection. Agents that declare none render the section's empty
    // state.
    const providerList = derived<{ id: string; label: string; isDefault: boolean }[]>(() => {
      const d = docAgent()
      if (!d || !Array.isArray(d.providers)) return []
      const def = d.default_provider || (d.providers.length > 0 ? d.providers[0] : '')
      return d.providers.map((id) => ({
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
      return a.permissionsTier || 'none'
    })

    const permLists = derived(() => {
      const p = agent()?.permissions
      return {
        allow: Array.isArray(p?.toolsAllow) ? p!.toolsAllow : [],
        ask: Array.isArray(p?.toolsAsk) ? p!.toolsAsk : [],
        deny: Array.isArray(p?.toolsDeny) ? p!.toolsDeny : [],
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
    void readActiveAgent(ctx.clients.ops).then((id) => activeAgentId.set(id))

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
    goWire: (s) => navigate(routes.studioAgentsIdWire({ id: s.agentId })),
    goSettings: (s) => navigate(routes.studioAgentsIdSettings({ id: s.agentId })),
    // Segmented tabs: the control swallows per-option clicks, so navigate off
    // its sui-change event instead. overview → detail root; else → sub-route.
    goTab: (s, value: string) => {
      const id = s.agentId
      navigate(value === 'overview' ? routes.studioAgentsId({ id }) : `/studio/agents/${id}/${value}`)
    },
    goBack: () => navigate('/studio/agents'),
    goSkill: (_s, id: string) => navigate(routes.studioSkillsId({ id })),

    activate: async (s) => {
      if (s.activating) return
      s.activating = true
      s.banner = null
      try {
        await writeSetting(s.clients.ops, 'shell.agent.active', s.agentId)
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
        await writeSetting(s.clients.ops, 'shell.agent.active', '')
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
