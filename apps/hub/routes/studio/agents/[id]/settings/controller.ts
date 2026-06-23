import { controller, signal, derived, effect, live, unwrap } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { navigate } from '@ship/deck/router'
import { routes } from '../../../../../routes.gen'

type DraftKey = 'allowDraft' | 'askDraft' | 'denyDraft'
type ArrKey = 'pAllow' | 'pAsk' | 'pDeny'

export default controller({
  scope: (ctx: ViewScope<Scope>) => {
    // AMBIENT `ops` client (host-owned). `agents.Settings` carries the
    // view-shaped agent + tiers + catalogs (camelCase: mcpServers,
    // toolPatterns). Providers are not on that projection, so they come from
    // the raw doc via `agents.Get`.
    const ops = ctx.clients.ops
    const { route } = ctx
    const agentId = derived(() => route.params.id() ?? '')
    const settings = live(() => ops.agents.settings({ id: agentId() }))
    const agent = derived(() => unwrap(settings)?.agent ?? null)
    const catalogs = derived(() => unwrap(settings)?.catalogs ?? null)
    const docCtx = live(() => ops.agents.get({ id: agentId(), scope: '' }))
    const docAgent = derived<{ providers?: string[]; default_provider?: string } | null>(() => {
      const r = unwrap(docCtx)
      if (!r) return null
      try {
        return (JSON.parse(r.docJson) as { agent?: { providers?: string[]; default_provider?: string } }).agent ?? null
      } catch {
        return null
      }
    })
    const providers = derived(() => {
      const d = docAgent()
      const list = Array.isArray(d?.providers) ? d!.providers : []
      const def = d?.default_provider
      return list.map((id) => ({
        id,
        label: id.length > 0 ? id[0].toUpperCase() + id.slice(1) : id,
        isDefault: id === def,
      }))
    })
    const pAllow = signal<string[]>([])
    const pAsk = signal<string[]>([])
    const pDeny = signal<string[]>([])
    const pPreset = signal('')
    const pDirty = signal(false)
    const pSaving = signal(false)
    const allowDraft = signal('')
    const askDraft = signal('')
    const denyDraft = signal('')
    const pickerOpen = signal(false)
    const pickerKind = signal('skill')
    const pickerSelected = signal<string[]>([])
    const _settStop = signal<(() => void) | null>(null)
    return { agentId, settings, agent, catalogs, providers, pAllow, pAsk, pDeny, pPreset, pDirty, pSaving, allowDraft, askDraft, denyDraft, pickerOpen, pickerKind, pickerSelected, _settStop }
  },
  on: {
    goWire: (s) => navigate(routes.studioAgentsIdWire({ id: s.agentId })),
    goTab: (s, value: string) => {
      const id = s.agentId
      navigate(value === 'overview' ? routes.studioAgentsId({ id }) : `/studio/agents/${id}/${value}`)
    },
    goBack: () => navigate(routes.studioAgents()),
    openPicker: (s, kind: string) => { s.pickerKind = kind; s.pickerSelected = []; s.pickerOpen = true },
    closePicker: (s) => { s.pickerOpen = false; s.pickerSelected = [] },
    togglePick: (s, id: string) => {
      const cur = s.pickerSelected
      s.pickerSelected = cur.includes(id) ? cur.filter((x: string) => x !== id) : [...cur, id]
    },
    savePicker: (s) => {
      const selected = s.pickerSelected
      if (!selected.length) { s.pickerOpen = false; return }
      const kind = s.pickerKind
      const id = s.agentId
      const path = kind === 'rule' ? 'rules' : kind === 'mcp' ? 'mcp' : 'skills'
      const sub = kind === 'mcp' ? 'servers' : 'refs'
      void s.clients.ops.agents.patch({
        id,
        scope: '',
        patchJson: JSON.stringify(selected.map((sid: string) => ({ op: 'add', path: `/${path}/${sub}/-`, value: sid }))),
      }).then(() => { s.pickerOpen = false; s.pickerSelected = []; window.location.reload() })
    },
    addPerm: (s, e: KeyboardEvent, kind: string) => {
      if (e.key !== 'Enter') return
      const draftKey: DraftKey = kind === 'allow' ? 'allowDraft' : kind === 'ask' ? 'askDraft' : 'denyDraft'
      const draft = s[draftKey].trim()
      if (!draft) return
      const arrKey: ArrKey = kind === 'allow' ? 'pAllow' : kind === 'ask' ? 'pAsk' : 'pDeny'
      const arr = s[arrKey]
      if (!arr.includes(draft)) { s[arrKey] = [...arr, draft]; s.pDirty = true }
      s[draftKey] = ''
      e.preventDefault()
    },
    commitPerm: (s, kind: string) => {
      const draftKey: DraftKey = kind === 'allow' ? 'allowDraft' : kind === 'ask' ? 'askDraft' : 'denyDraft'
      const draft = s[draftKey].trim()
      if (!draft) return
      const arrKey: ArrKey = kind === 'allow' ? 'pAllow' : kind === 'ask' ? 'pAsk' : 'pDeny'
      const arr = s[arrKey]
      if (!arr.includes(draft)) { s[arrKey] = [...arr, draft]; s.pDirty = true }
      s[draftKey] = ''
    },
    removePerm: (s, kind: string, pattern: string) => {
      const arrKey: ArrKey = kind === 'allow' ? 'pAllow' : kind === 'ask' ? 'pAsk' : 'pDeny'
      s[arrKey] = s[arrKey].filter((x: string) => x !== pattern)
      s.pDirty = true
    },
    savePermissions: (s) => {
      s.pSaving = true
      const id = s.agentId
      void s.clients.ops.agents.patch({
        id,
        scope: '',
        patchJson: JSON.stringify([{ op: 'add', path: '/permissions', value: { preset: s.pPreset, tools_allow: s.pAllow, tools_ask: s.pAsk, tools_deny: s.pDeny } }]),
      }).then(() => { s.pDirty = false; s.pSaving = false; window.location.reload() })
        .catch(() => { s.pSaving = false })
    },
  },
  mount: (state, _el) => {
    const stop = effect(() => {
      const a = state.agent as { permissions?: { toolsAllow?: string[]; toolsAsk?: string[]; toolsDeny?: string[]; preset?: string } } | null
      if (!a || state.pDirty) return
      const perms = a.permissions ?? {}
      state.pAllow = perms.toolsAllow ?? []
      state.pAsk = perms.toolsAsk ?? []
      state.pDeny = perms.toolsDeny ?? []
      state.pPreset = perms.preset ?? ''
    })
    state._settStop = stop
  },
  unmount: (state) => {
    state._settStop?.()
  },
})
