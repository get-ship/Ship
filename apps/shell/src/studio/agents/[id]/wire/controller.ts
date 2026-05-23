import { controller, signal, derived, effect, live, unwrap } from '@ship/sui-runtime'
import type { MutableState } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

// `on:` handlers receive the writable state proxy; annotate explicitly
// since the OnSection union blocks contextual inference.
type S = MutableState<Scope>
// `navigate` is injected onto the proxy by the runtime mount context but
// is not a template-declared scope key, so it isn't on `MutableState`.
type Nav = { navigate: (path: string) => void }

export default controller<Scope>({
  scope: ({ route, ship }) => {
    const agentId = derived(() => route.params.id() ?? '')
    const wireCtx = live(() => ship.agents.wire({ id: agentId() }))
    const wireData = derived(() => unwrap(wireCtx as any) ?? null)
    const selectedKind = signal('agent')
    const selectedId = signal('')
    const pickerOpen = signal(false)
    const pickerKind = signal('skill')
    const pickerSelected = signal<string[]>([])
    const _wireStop = signal<(() => void) | null>(null)
    return { agentId, wireCtx, wireData, selectedKind, selectedId, pickerOpen, pickerKind, pickerSelected, _wireStop }
  },
  on: {
    selectNode: (s: S, kind: string, id: string) => { s.selectedKind = kind; s.selectedId = id },
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
    goSettings: (s: S) => (s as unknown as Nav).navigate(`/studio/agents/${s.agentId as string}/settings`),
    goTab: (s: S, value: string) => {
      const id = s.agentId as string
      return (s as unknown as Nav).navigate(value === 'overview' ? `/studio/agents/${id}` : `/studio/agents/${id}/${value}`)
    },
    goBack: (s: S) => (s as unknown as Nav).navigate('/studio/agents'),
  },
  mount: (state, _el) => {
    const stop = effect(() => {
      const d = state.wireData as { agent?: { id: string } } | null
      if (d?.agent?.id && !(state.selectedId as string)) {
        state.selectedId = d.agent.id
      }
    })
    state._wireStop = stop
  },
  unmount: (state) => {
    const fn = state._wireStop as (() => void) | null | undefined
    fn?.()
  },
})
