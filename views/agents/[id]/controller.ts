import { controller, derived, effect, live, signal, unwrap } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

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

type SectionId =
  | 'description'
  | 'skills'
  | 'mcp'
  | 'rules'
  | 'permissions'
  | 'providers'
  | 'source'

const SECTIONS: SectionId[] = [
  'description', 'skills', 'mcp', 'rules', 'permissions', 'providers', 'source',
]

function readSectionFromUrl(): SectionId {
  if (typeof location === 'undefined') return 'description'
  const params = new URLSearchParams(location.search)
  const s = params.get('section')
  return (SECTIONS as string[]).includes(s ?? '') ? (s as SectionId) : 'description'
}

function writeSectionToUrl(section: SectionId): void {
  if (typeof window === 'undefined') return
  const url = new URL(window.location.href)
  if (section === 'description') url.searchParams.delete('section')
  else url.searchParams.set('section', section)
  window.history.replaceState(null, '', url.toString())
}

async function readActiveAgent(): Promise<string> {
  try {
    const resp = await fetch('/api/v1/settings/shell.agent.active', { credentials: 'same-origin' })
    if (!resp.ok) return ''
    const body = (await resp.json()) as { data?: unknown }
    const value = body.data
    if (typeof value === 'string') return value
    if (value && typeof value === 'object' && 'value' in value && typeof (value as { value?: unknown }).value === 'string') {
      return (value as { value: string }).value
    }
    return ''
  } catch {
    return ''
  }
}

export default controller<Scope>({
  scope: ({ route, ship }) => {
    const agentId = derived(() => route.params.id() ?? '')
    const wireCtx = live(() => ship.agents.wire({ id: agentId() }))
    const wireData = derived(() => (unwrap(wireCtx as unknown) ?? null) as WireResp)

    const agent = derived<AgentDetail | null>(() => {
      const d = wireData()
      return (d && d.agent) ? d.agent : null
    })

    const notFound = derived(() => {
      const ctx = unwrap(wireCtx as unknown)
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
    const hasInlineRules = derived(() => {
      const a = agent()
      return !!(a && typeof a.inline_rules === 'string' && a.inline_rules.trim().length > 0)
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

    // Edit state — drafts seeded from the loaded agent; baselines track
    // last-saved values so the dirty flag stays honest across re-renders.
    const descDraft = signal('')
    const descBaseline = signal('')
    const rulesDraft = signal('')
    const rulesBaseline = signal('')
    const saving = signal(false)
    const banner = signal<Banner>(null)

    // Active-agent — workspace-scoped via the `shell.agent.active` setting.
    // Polled on mount + refreshed after every activate/deactivate. Could be
    // pushed via SSE on `config.changed` later; today the activate handler
    // emits the event so other views see it.
    const activeAgentId = signal('')
    const activating = signal(false)
    void readActiveAgent().then((id) => activeAgentId.set(id))

    const isActive = derived(() => {
      const id = agentId()
      return id !== '' && activeAgentId() === id
    })

    // Seed editor baselines + drafts whenever the loaded agent changes.
    effect(() => {
      const a = agent()
      if (!a) return
      const desc = a.description ?? ''
      const rules = a.inline_rules ?? ''
      descBaseline.set(desc)
      descDraft.set(desc)
      rulesBaseline.set(rules)
      rulesDraft.set(rules)
      banner.set(null)
    })

    const descDirty = derived(() => descDraft() !== descBaseline())
    const rulesDirty = derived(() => rulesDraft() !== rulesBaseline())

    const openSection = signal<SectionId>(readSectionFromUrl())

    // Keep section state in sync with browser back/forward, which lets
    // deep-linked URLs like /agents/foo?section=permissions land on the
    // right tab even after popstate.
    effect(() => {
      if (typeof window === 'undefined') return
      const onPop = () => openSection.set(readSectionFromUrl())
      window.addEventListener('popstate', onPop)
      window.addEventListener('sui:navigate', onPop)
      return () => {
        window.removeEventListener('popstate', onPop)
        window.removeEventListener('sui:navigate', onPop)
      }
    })

    return {
      agentId, wireCtx, wireData, agent, notFound, description,
      skillsList, mcpList, rulesList, hasInlineRules, providerList,
      stats, permPreset, permLists, permTotalCount, sourcePath,
      descDraft, descBaseline, rulesDraft, rulesBaseline,
      descDirty, rulesDirty, saving, banner,
      activeAgentId, activating, isActive,
      openSection,
    }
  },
  on: {
    goWire: ({ navigate, agentId }) => navigate(`/agents/${agentId}/wire`),
    goSettings: ({ navigate, agentId }) => navigate(`/agents/${agentId}/settings`),
    goBack: ({ navigate }) => navigate('/agents'),
    goSkill: ({ navigate }, id: string) => navigate(`/skills/${id}`),

    toggleSection: (s, id: SectionId) => {
      const next = s.openSection === id ? 'description' : id
      s.openSection = next
      writeSectionToUrl(next as SectionId)
    },

    onDescInput: (s, e: Event) => {
      const target = e.target as HTMLTextAreaElement & { value?: string }
      const detail = (e as CustomEvent<{ value?: string }>).detail
      const next = typeof detail?.value === 'string' ? detail.value : target?.value ?? ''
      s.descDraft = next
    },
    resetDesc: (s) => { s.descDraft = s.descBaseline },
    saveDesc: async (s) => {
      if (s.saving || !s.descDirty) return
      s.saving = true
      s.banner = null
      const next = s.descDraft as string
      try {
        const resp = await fetch(`/api/v1/agents/${encodeURIComponent(s.agentId as string)}`, {
          method: 'PATCH',
          credentials: 'same-origin',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify([
            { op: 'replace', path: '/agent/description', value: next },
          ]),
        })
        if (!resp.ok) {
          const text = await resp.text().catch(() => '')
          s.banner = { ok: false, msg: text || `Save failed (${resp.status})` }
          return
        }
        s.descBaseline = next
        s.banner = { ok: true, msg: 'Description saved.' }
      } catch (err) {
        s.banner = { ok: false, msg: (err as Error).message ?? 'Save failed.' }
      } finally {
        s.saving = false
      }
    },

    onRulesInput: (s, e: Event) => {
      const target = e.target as HTMLTextAreaElement & { value?: string }
      const detail = (e as CustomEvent<{ value?: string }>).detail
      const next = typeof detail?.value === 'string' ? detail.value : target?.value ?? ''
      s.rulesDraft = next
    },
    resetRules: (s) => { s.rulesDraft = s.rulesBaseline },
    saveRules: async (s) => {
      if (s.saving || !s.rulesDirty) return
      s.saving = true
      s.banner = null
      const next = s.rulesDraft as string
      // `add` doubles as "create or replace" for json-patch object members,
      // which avoids a 422 when `rules` or `rules.inline` doesn't yet exist
      // on the document.
      try {
        const resp = await fetch(`/api/v1/agents/${encodeURIComponent(s.agentId as string)}`, {
          method: 'PATCH',
          credentials: 'same-origin',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify([
            { op: 'add', path: '/rules', value: { inline: next } },
          ]),
        })
        if (!resp.ok) {
          const text = await resp.text().catch(() => '')
          s.banner = { ok: false, msg: text || `Save failed (${resp.status})` }
          return
        }
        s.rulesBaseline = next
        s.banner = { ok: true, msg: 'Rules saved.' }
      } catch (err) {
        s.banner = { ok: false, msg: (err as Error).message ?? 'Save failed.' }
      } finally {
        s.saving = false
      }
    },

    activate: async (s) => {
      if (s.activating) return
      s.activating = true
      s.banner = null
      try {
        const resp = await fetch('/api/v1/settings', {
          method: 'POST',
          credentials: 'same-origin',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify({ key: 'shell.agent.active', value: s.agentId }),
        })
        if (!resp.ok) {
          const text = await resp.text().catch(() => '')
          s.banner = { ok: false, msg: text || `Activate failed (${resp.status})` }
          return
        }
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
        const resp = await fetch('/api/v1/settings', {
          method: 'POST',
          credentials: 'same-origin',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify({ key: 'shell.agent.active', value: '' }),
        })
        if (!resp.ok) {
          const text = await resp.text().catch(() => '')
          s.banner = { ok: false, msg: text || `Deactivate failed (${resp.status})` }
          return
        }
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
