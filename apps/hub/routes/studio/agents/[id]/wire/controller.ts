import { controller, signal, derived, effect, live, unwrap } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { navigate } from '@ship/deck/router'
import { routes } from '../../../../../routes.gen'

export default controller({
  scope: (ctx: ViewScope<Scope>) => {
    const ops = ctx.clients.ops
    const { route } = ctx
    const agentId = derived(() => route.params.id() ?? '')
    // Unary `agents.Wire` over the ambient `ops` client; `live()` wraps the
    // promise so the template's `{#async}` block drives loading/error/ok.
    // Response is `AgentWireResponse` { agent, nodes, edges, catalogs } with
    // camelCase fields (mcpServers, pathD).
    const wireCtx = live(() => ops.agents.wire({ id: agentId() }))
    const wireData = derived(() => unwrap(wireCtx) ?? null)
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
    goSettings: (s) => navigate(routes.studioAgentsIdSettings({ id: s.agentId })),
    goTab: (s, value: string) => {
      const id = s.agentId
      navigate(value === 'overview' ? routes.studioAgentsId({ id }) : `/studio/agents/${id}/${value}`)
    },
    goBack: () => navigate(routes.studioAgents()),
  },
  mount: (state, _el) => {
    const stop = effect(() => {
      const d = state.wireData
      if (d?.agent?.id && !state.selectedId) {
        state.selectedId = d.agent.id
      }
    })
    state._wireStop = stop
  },
  unmount: (state) => {
    state._wireStop?.()
  },
})
