import { controller, signal, derived, effect, live, unwrap } from '@ship/sui-runtime'
import type { MutableState } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

// `on:`/`mount` handlers receive the writable state proxy; annotate
// explicitly since the OnSection union blocks contextual inference.
type S = MutableState<Scope>
// `navigate` is injected onto the proxy by the runtime mount context but
// is not a template-declared scope key, so it isn't on `MutableState`.
type Nav = { navigate: (path: string) => void }
type DraftKey = 'allowDraft' | 'askDraft' | 'denyDraft'
type ArrKey = 'pAllow' | 'pAsk' | 'pDeny'

export default controller<Scope>({
  scope: ({ ship, route }) => {
    const agentId = derived(() => route.params.id() ?? '')
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const s = ship as any
    const settings = live(() => s.agents.settings({ id: agentId() }))
    const agent = derived(() => unwrap(settings)?.agent ?? null)
    const catalogs = derived(() => unwrap(settings)?.catalogs ?? null)
    const providers = derived(() => {
      const a = unwrap(settings) as { agent?: { providers?: string[]; default_provider?: string } } | undefined
      const list = a?.agent?.providers ?? []
      const def = a?.agent?.default_provider
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
    goWire: (s: S) => (s as unknown as Nav).navigate(`/studio/agents/${s.agentId as string}/wire`),
    goTab: (s: S, value: string) => {
      const id = s.agentId as string
      return (s as unknown as Nav).navigate(value === 'overview' ? `/studio/agents/${id}` : `/studio/agents/${id}/${value}`)
    },
    goBack: (s: S) => (s as unknown as Nav).navigate('/studio/agents'),
    openPicker: (s: S, kind: string) => { s.pickerKind = kind; s.pickerSelected = []; s.pickerOpen = true },
    closePicker: (s: S) => { s.pickerOpen = false; s.pickerSelected = [] },
    togglePick: (s: S, id: string) => {
      const cur = s.pickerSelected as string[]
      s.pickerSelected = cur.includes(id) ? cur.filter(x => x !== id) : [...cur, id]
    },
    savePicker: (s: S) => {
      const selected = s.pickerSelected as string[]
      if (!selected.length) { s.pickerOpen = false; return }
      const kind = s.pickerKind as string
      const id = s.agentId as string
      const path = kind === 'rule' ? 'rules' : kind === 'mcp' ? 'mcp' : 'skills'
      const sub = kind === 'mcp' ? 'servers' : 'refs'
      fetch(`/api/v1/agents/${id}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify(selected.map(sid => ({ op: 'add', path: `/${path}/${sub}/-`, value: sid }))),
      }).then(() => { s.pickerOpen = false; s.pickerSelected = []; window.location.reload() })
    },
    addPerm: (s: S, e: KeyboardEvent, kind: string) => {
      if (e.key !== 'Enter') return
      const draftKey: DraftKey = kind === 'allow' ? 'allowDraft' : kind === 'ask' ? 'askDraft' : 'denyDraft'
      const draft = (s[draftKey] as string).trim()
      if (!draft) return
      const arrKey: ArrKey = kind === 'allow' ? 'pAllow' : kind === 'ask' ? 'pAsk' : 'pDeny'
      const arr = s[arrKey] as string[]
      if (!arr.includes(draft)) { s[arrKey] = [...arr, draft]; s.pDirty = true }
      s[draftKey] = ''
      e.preventDefault()
    },
    commitPerm: (s: S, kind: string) => {
      const draftKey: DraftKey = kind === 'allow' ? 'allowDraft' : kind === 'ask' ? 'askDraft' : 'denyDraft'
      const draft = (s[draftKey] as string).trim()
      if (!draft) return
      const arrKey: ArrKey = kind === 'allow' ? 'pAllow' : kind === 'ask' ? 'pAsk' : 'pDeny'
      const arr = s[arrKey] as string[]
      if (!arr.includes(draft)) { s[arrKey] = [...arr, draft]; s.pDirty = true }
      s[draftKey] = ''
    },
    removePerm: (s: S, kind: string, pattern: string) => {
      const arrKey: ArrKey = kind === 'allow' ? 'pAllow' : kind === 'ask' ? 'pAsk' : 'pDeny'
      s[arrKey] = (s[arrKey] as string[]).filter(x => x !== pattern)
      s.pDirty = true
    },
    savePermissions: (s: S) => {
      s.pSaving = true
      const id = s.agentId as string
      fetch(`/api/v1/agents/${id}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify([{ op: 'add', path: '/permissions', value: { preset: s.pPreset, tools_allow: s.pAllow, tools_ask: s.pAsk, tools_deny: s.pDeny } }]),
      }).then(() => { s.pDirty = false; s.pSaving = false; window.location.reload() })
        .catch(() => { s.pSaving = false })
    },
  },
  mount: (state, _el) => {
    const stop = effect(() => {
      const a = state.agent as { permissions?: { tools_allow?: string[]; tools_ask?: string[]; tools_deny?: string[]; preset?: string } } | null
      if (!a || (state.pDirty as boolean)) return
      const perms = a.permissions ?? {}
      state.pAllow = perms.tools_allow ?? []
      state.pAsk = perms.tools_ask ?? []
      state.pDeny = perms.tools_deny ?? []
      state.pPreset = perms.preset ?? ''
    })
    state._settStop = stop
  },
  unmount: (state) => {
    const fn = state._settStop as (() => void) | null | undefined
    fn?.()
  },
})
