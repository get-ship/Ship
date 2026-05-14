import { controller, signal, derived, effect, live, unwrap } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: ({ route, ship }) => {
    const agentId = derived(() => route().params.id)
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
    selectNode: (s, kind: string, id: string) => { s.selectedKind = kind; s.selectedId = id },
    openPicker: (s, kind: string) => { s.pickerKind = kind; s.pickerSelected = []; s.pickerOpen = true },
    closePicker: (s) => { s.pickerOpen = false; s.pickerSelected = [] },
    togglePick: (s, id: string) => {
      const cur = s.pickerSelected as string[]
      s.pickerSelected = cur.includes(id) ? cur.filter(x => x !== id) : [...cur, id]
    },
    savePicker: (s) => {
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
    goSettings: ({ navigate, agentId }) => navigate(`/agents/${agentId}/settings`),
    goBack: ({ navigate }) => navigate('/agents'),
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
