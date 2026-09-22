<controller>
import { controller, signal, derived, effect, live, unwrap } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { navigate } from '@ship/deck-router'
import { routes } from '../../../../../routes.gen'

export default controller({
  scope: (ctx: ViewScope<Scope>) => {
    const ops = ctx.ops
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
</controller>

<div data-test="studio-view">
  <ship-studio-rail></ship-studio-rail>
  <main data-test="studio-content">

  <while-loading of="wireCtx">
  <spinner size="md" centered></spinner>

  </while-loading><while-error as="err" of="wireCtx">
  <p style="padding:var(--space-4); color:var(--destructive)">{err.message}</p>

  </while-error><if cond={wireCtx().kind === 'ok'}>
<if cond={wireData()}>
  <ship-view-chrome title={wireData().agent.name}>
    <toggle-group variant="segmented" type="single" slot="actions" value="wire" size="sm" on:sail-change={(e) => goTab(e.detail.value)}>
      <toggle value="overview">Overview</toggle>
      <toggle value="wire">Wire</toggle>
      <toggle value="settings">Settings</toggle>
    </toggle-group>
    <button slot="actions" variant="ghost" size="sm" on:click={goBack}>
      <icon slot="leading" name="chevron-left" size="xs"></icon>
      Agents
    </button>
  </ship-view-chrome>

  <div>

    <aside>
      <div>
        <icon name="zap" size="xs"></icon>Skills
        <span>{wireData().agent.skills.length}</span>
        <button variant="ghost" size="sm" aria-label="Add skill" on:click={() => openPicker('skill')}>
          <icon name="plus" size="xs"></icon>
        </button>
      </div>
      <if cond={wireData().agent.skills.length === 0}>
        <div style="font-style:italic">no skills wired</div>
      <else>
        <for as="s" of={wireData().agent.skills} key={s}>
          <div>
            <icon name="check-circle" size="xs" style="color:var(--tone-success-fg)"></icon>
            <span>{s}</span>
            <span style="margin-left:auto; font-size:var(--text-xs)">wired</span>
          </div>
        </for>
      </else></if>
      <div style="margin-top:16px">
        <icon name="terminal" size="xs"></icon>MCP Servers
        <span>{wireData().agent.mcpServers.length}</span>
        <button variant="ghost" size="sm" aria-label="Add MCP server" on:click={() => openPicker('mcp')}>
          <icon name="plus" size="xs"></icon>
        </button>
      </div>
      <if cond={wireData().agent.mcpServers.length === 0}>
        <div style="font-style:italic">no servers</div>
      <else>
        <for as="m" of={wireData().agent.mcpServers} key={m}>
          <div>
            <icon name="check-circle" size="xs" style="color:var(--tone-info-fg)"></icon>
            <span>{m}</span>
          </div>
        </for>
      </else></if>
      <div style="margin-top:16px">
        <icon name="file-text" size="xs"></icon>Rules
        <span>{wireData().agent.rules.length}</span>
        <button variant="ghost" size="sm" aria-label="Add rule" on:click={() => openPicker('rule')}>
          <icon name="plus" size="xs"></icon>
        </button>
      </div>
      <for as="r" of={wireData().agent.rules} key={r}>
        <div><span>{r}</span></div>
      </for>
      <div><strong style="color:var(--foreground)">Wire view.</strong> Click any node to inspect. Use + on a section to add.</div>
    </aside>

    <div>
      <svg viewBox="0 0 1000 600" preserveAspectRatio="none">
        <for as="e" of={wireData().edges} key={e.pathD}>
          <path d={e.pathD}
            stroke={'var(--tone-' + (e.kind === 'skill' ? 'success' : e.kind === 'mcp' ? 'info' : e.kind === 'rule' ? 'warning' : e.kind === 'perm' ? 'destructive' : 'accent') + '-fg)'}
            stroke-width="1.3" stroke-opacity="0.55" fill="none"
            stroke-dasharray={e.dashed ? '4 3' : null} />
        </for>
      </svg>
      <for as="n" of={wireData().nodes} key={n.id}>
        <div
          style:left={`${n.x / 10}%`} style:top={`${n.y / 6}%`}
          style:width={`${n.w / 10}%`} style:min-width={`${n.w * 0.7}px`}
          data-selected={selectedKind() === n.kind && selectedId() === n.id}
          on:click={() => selectNode(n.kind, n.id)}>
          <div><span></span>
            <span>{n.kind === 'agent' ? 'AGENT' : n.kind === 'skill' ? 'SKILL' : n.kind === 'mcp' ? 'MCP SERVER' : n.kind === 'rule' ? 'RULES' : n.kind === 'perm' ? 'PERMISSIONS' : 'PROVIDER'}</span>
          </div>
          <if cond={n.kind === 'prov'}>
            <div style="display:flex; align-items:center; gap:8px">
              <span>{n.id.slice(0,2).toUpperCase()}</span>
              <div style="min-width:0"><div>{n.title}</div><div>{n.sub}</div></div>
            </div>
          <else-if cond={n.kind === 'agent'}>
            <div style="display:flex; align-items:center; gap:10px">
              <avatar size="md" shape="square" identity-hue={wireData().agent.id} initials={wireData().agent.letter} name={wireData().agent.name}></avatar>
              <div style="min-width:0"><div>{n.title}</div><div>{n.sub}</div></div>
            </div>
          </else-if><else>
            <div>{n.title}</div><div>{n.sub}</div>
          </else></if>
        </div>
      </for>
      <div>
        <div><i style="background:var(--tone-success-fg)"></i>skill</div>
        <div><i style="background:var(--tone-info-fg)"></i>mcp</div>
        <div><i style="background:var(--tone-warning-fg)"></i>rule</div>
        <div><i style="background:var(--tone-destructive-fg)"></i>perm</div>
        <div><i style="background:var(--tone-accent-fg)"></i>provider</div>
      </div>
    </div>

    <aside>
      <match on={selectedKind()}>
      <case of={'agent'}>
        <div><div><div>Agent</div><div>{wireData().agent.name}</div></div></div>
        <div><label>ID</label><div>{wireData().agent.id}</div></div>
        <div><label>Description</label><div style="font-size:var(--text-sm); line-height:1.5">{wireData().agent.description || 'No description'}</div></div>
        <div><label>Source</label><div>{wireData().agent.source}</div></div>
        <p>Select any peripheral node on the canvas to inspect.</p>
        <button size="sm" variant="outline" style="margin-top:var(--space-3); width:100%" on:click={goSettings}>
          <icon slot="leading" name="edit" size="xs"></icon>Edit settings
        </button>
      </case><case of={'skill'}>
        <div>
          <icon name="zap" size="sm" style="color:var(--tone-success-fg)"></icon>
          <div><div>Skill</div><div>{selectedId()}</div></div>
        </div>
        <div><label>Scope</label><chip tone="success" variant="soft" size="sm">project</chip></div>
        <div><label>Source</label><div>.ship/skills/{selectedId()}/SKILL.md</div></div>
      </case><case of={'mcp'}>
        <div>
          <icon name="terminal" size="sm" style="color:var(--tone-info-fg)"></icon>
          <div><div>MCP server</div><div>{selectedId()}</div></div>
        </div>
        <div><label>Source</label><div>.ship/mcp.jsonc → servers.{selectedId()}</div></div>
      </case><case of={'rule'}>
        <div>
          <icon name="file-text" size="sm" style="color:var(--tone-warning-fg)"></icon>
          <div><div>Rules bundle</div><div>{wireData().agent.rules.length} file{wireData().agent.rules.length !== 1 ? 's' : ''}</div></div>
        </div>
        <div><label>Files</label>
          <for as="r" of={wireData().agent.rules} key={r}>
            <div style="padding:4px 8px; background:var(--muted); border-radius:4px; margin-bottom:4px">{r}</div>
          </for>
        </div>
      </case><case of={'perm'}>
        <div>
          <icon name="lock" size="sm" style="color:var(--tone-destructive-fg)"></icon>
          <div><div>Permissions</div><div>{wireData().agent.permissions?.preset || 'no preset'}</div></div>
        </div>
        <button size="sm" variant="outline" style="margin-top:var(--space-3); width:100%" on:click={goSettings}>
          <icon slot="leading" name="edit" size="xs"></icon>Edit permissions
        </button>
      </case><case of={'prov'}>
        <div>
          <span>{selectedId().slice(0,2).toUpperCase()}</span>
          <div><div>Provider</div><div>{selectedId()}</div></div>
        </div>
        <p>Provider-specific settings are compiled on top of project defaults.</p>
      </case><default>
        <p>Select a node to inspect.</p>
      </default></match>
    </aside>

    <dialog open={pickerOpen()} size="md" on:sail-close={closePicker}>
      <span slot="header">Add {pickerKind() === 'skill' ? 'skill' : pickerKind() === 'mcp' ? 'MCP server' : 'rule'}</span>
      <div>
        <if cond={wireData().catalogs[pickerKind()]?.length === 0}>
          <empty-state variant="placeholder" size="sm">
            <span slot="title">No {pickerKind()}s to add</span>
          </empty-state>
        <else>
          <column>
            <for as="entry" of={(wireData().catalogs[pickerKind()] ?? [])} key={entry.id}>
              <label>
                <input type="checkbox" value={entry.id}
                  checked={pickerSelected().includes(entry.id)}
                  on:change={() => togglePick(entry.id)} />
                <span>
                  <span>{entry.name}</span>
                  <span>{entry.description || entry.id}</span>
                </span>
              </label>
            </for>
          </column>
        </else></if>
      </div>
      <div slot="footer">
        <span style="flex:1; font-size:var(--text-xs); color:var(--muted-foreground)">{pickerSelected().length} selected</span>
        <button variant="ghost" size="sm" on:click={closePicker}>Cancel</button>
        <button tone="primary" variant="solid" size="sm"
          disabled={pickerSelected().length === 0}
          on:click={savePicker}>Add</button>
      </div>
    </dialog>

  </div>
</if>
</if>
  </main>
</div>
