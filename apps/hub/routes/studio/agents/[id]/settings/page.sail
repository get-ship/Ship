<controller>
import { controller, signal, derived, effect, live, unwrap } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { navigate } from '@ship/deck-router'
import { routes } from '../../../../../routes.gen'

type DraftKey = 'allowDraft' | 'askDraft' | 'denyDraft'
type ArrKey = 'pAllow' | 'pAsk' | 'pDeny'

export default controller((ctx: ViewScope<Scope>) => {

    // AMBIENT `ops` client (host-owned). `agents.Settings` carries the
    // view-shaped agent + tiers + catalogs (camelCase: mcpServers,
    // toolPatterns). Providers are not on that projection, so they come from
    // the raw doc via `agents.Get`.
    const ops = ctx.ops
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
    
    const goWire = (s) => navigate(routes.studioAgentsIdWire({ id: s.agentId }))
    const goTab = (value: string) => {
      const id = agentId
      navigate(value === 'overview' ? routes.studioAgentsId({ id }) : `/studio/agents/${id}/${value}`)
    }
    const goBack = () => navigate(routes.studioAgents())
    const openPicker = (kind: string) => { pickerKind = kind; pickerSelected = []; pickerOpen = true }
    const closePicker = (s) => { s.pickerOpen = false; s.pickerSelected = [] }
    const togglePick = (id: string) => {
      const cur = pickerSelected
      pickerSelected = cur.includes(id) ? cur.filter((x: string) => x !== id) : [...cur, id]
    }
    const savePicker = (s) => {
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
    }
    const addPerm = (e: KeyboardEvent, kind: string) => {
      if (e.key !== 'Enter') return
      const draftKey: DraftKey = kind === 'allow' ? 'allowDraft' : kind === 'ask' ? 'askDraft' : 'denyDraft'
      const draft = s[draftKey].trim()
      if (!draft) return
      const arrKey: ArrKey = kind === 'allow' ? 'pAllow' : kind === 'ask' ? 'pAsk' : 'pDeny'
      const arr = s[arrKey]
      if (!arr.includes(draft)) { s[arrKey] = [...arr, draft]; pDirty = true }
      s[draftKey] = ''
      e.preventDefault()
    }
    const commitPerm = (kind: string) => {
      const draftKey: DraftKey = kind === 'allow' ? 'allowDraft' : kind === 'ask' ? 'askDraft' : 'denyDraft'
      const draft = s[draftKey].trim()
      if (!draft) return
      const arrKey: ArrKey = kind === 'allow' ? 'pAllow' : kind === 'ask' ? 'pAsk' : 'pDeny'
      const arr = s[arrKey]
      if (!arr.includes(draft)) { s[arrKey] = [...arr, draft]; pDirty = true }
      s[draftKey] = ''
    }
    const removePerm = (kind: string, pattern: string) => {
      const arrKey: ArrKey = kind === 'allow' ? 'pAllow' : kind === 'ask' ? 'pAsk' : 'pDeny'
      s[arrKey] = s[arrKey].filter((x: string) => x !== pattern)
      pDirty = true
    }
    const savePermissions = (s) => {
      s.pSaving = true
      const id = s.agentId
      void s.clients.ops.agents.patch({
        id,
        scope: '',
        patchJson: JSON.stringify([{ op: 'add', path: '/permissions', value: { preset: s.pPreset, tools_allow: s.pAllow, tools_ask: s.pAsk, tools_deny: s.pDeny } }]),
      }).then(() => { s.pDirty = false; s.pSaving = false; window.location.reload() })
        .catch(() => { s.pSaving = false })
    }
    return { agentId, settings, agent, catalogs, providers, pAllow, pAsk, pDeny, pPreset, pDirty, pSaving, allowDraft, askDraft, denyDraft, pickerOpen, pickerKind, pickerSelected, _settStop, goWire, goTab, goBack, openPicker, closePicker, togglePick, savePicker, addPerm, commitPerm, removePerm, savePermissions }
})
</controller>

<div data-test="studio-view">
  <ship-studio-rail></ship-studio-rail>
  <main data-test="studio-content">

  <while-loading of="settings">
  <spinner size="md" centered></spinner>

  </while-loading><while-error as="err" of="settings">
  <p style="padding:var(--space-4); color:var(--destructive)">{err.message}</p>

  </while-error><if cond={settings().kind === 'ok'}>
<if cond={agent()}>
  <ship-view-chrome title={agent().name}>
    <toggle-group variant="segmented" type="single" slot="actions" value="settings" size="sm" on:sail-change={(e) => goTab(e.detail.value)}>
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
    <div>
      <column gap="4">

        <card padding="none">
          <div slot="header">
            <h2><icon name="zap"></icon>Skills<span>{(agent().skills ?? []).length} attached</span></h2>
            <button variant="outline" size="sm" on:click={() => openPicker('skill')}><icon slot="leading" name="plus"></icon>Add</button>
          </div>
          <div>
            <if cond={(agent().skills ?? []).length === 0}>
              <div>No skills attached. Edit <code style="font-family:var(--font-mono);font-size:var(--text-xs)">.ship/agents/{agent().id}.jsonc</code> to add one.</div>
            <else>
              <row>
                <for as="s" of={(agent().skills ?? [])} key={s}>
                  <div style="display:flex;flex-direction:row">
                    <span>{s.slice(0,2)}</span>
                    <span>{s}</span>
                    <span>project</span>
                  </div>
                </for>
              </row>
            </else></if>
          </div>
        </card>

        <card padding="none">
          <div slot="header">
            <h2>
              <icon><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="7" height="7" x="3" y="3" rx="1"/><rect width="7" height="7" x="14" y="3" rx="1"/><rect width="7" height="7" x="14" y="14" rx="1"/><rect width="7" height="7" x="3" y="14" rx="1"/></svg></icon>
              MCP Servers<span>{(agent().mcpServers ?? []).length} attached</span>
            </h2>
            <button variant="outline" size="sm" on:click={() => openPicker('mcp')}><icon slot="leading" name="plus"></icon>Add</button>
          </div>
          <div>
            <if cond={(agent().mcpServers ?? []).length === 0}>
              <div>No MCP servers connected. Edit <code style="font-family:var(--font-mono);font-size:var(--text-xs)">.ship/agents/{agent().id}.jsonc</code> to add one.</div>
            <else>
              <row>
                <for as="m" of={(agent().mcpServers ?? [])} key={m}>
                  <div style="display:flex;flex-direction:row">
                    <span>{m.slice(0,2)}</span>
                    <span>{m}</span>
                    <span>server</span>
                  </div>
                </for>
              </row>
            </else></if>
          </div>
        </card>

        <card padding="none">
          <div slot="header">
            <h2><icon name="lock"></icon>Permissions</h2>
            <button variant="outline" size="sm"
              disabled={!pDirty() || pSaving()}
              on:click={savePermissions}>
              <icon slot="leading" name="check"></icon>Save
            </button>
          </div>
          <div>
            <field style="margin-bottom:var(--space-4)">
              <label>Preset</label>
              <select size="sm" value={pPreset()} on:sail-change={(e) => { pPreset.set(e.detail.value); pDirty.set(true) }}>
                <for as="p" of={(catalogs()?.presets ?? [])} key={p.id}>
                  <option value={p.id} selected={p.id === pPreset()}>{p.name}</option>
                </for>
              </select>
            </field>
            <datalist id="perm-tool-catalog">
              <for as="t" of={(catalogs()?.toolPatterns ?? [])} key={t.pattern}>
                <option value={t.pattern}>{t.label}</option>
              </for>
            </datalist>
            <div>
              <for as="col" of={[['allow','allowDraft','Allow','check'],['ask','askDraft','Ask','info'],['deny','denyDraft','Deny','x']]} key={col[0]}>
                <div data-perm-tone={col[0]}>
                  <div>
                    <icon name={col[3]}></icon>{col[2]}
                    <span>{col[0] === 'allow' ? pAllow().length : col[0] === 'ask' ? pAsk().length : pDeny().length}</span>
                  </div>
                  <div>
                    <input type="text" list="perm-tool-catalog" placeholder="Tool pattern…"
                      value={col[0] === 'allow' ? allowDraft() : col[0] === 'ask' ? askDraft() : denyDraft()}
                      on:input={(e) => (col[0] === 'allow' ? allowDraft : col[0] === 'ask' ? askDraft : denyDraft).set(e.target.value)}
                      on:keydown={(e) => addPerm(e, col[0])} />
                    <button variant="ghost" size="sm" aria-label="Add"
                      disabled={!(col[0] === 'allow' ? allowDraft() : col[0] === 'ask' ? askDraft() : denyDraft()).trim()}
                      on:click={() => commitPerm(col[0])}>
                      <icon name="plus" size="xs"></icon>
                    </button>
                  </div>
                  <div>
                    <if cond={pDirty()}>
                      <span>Unsaved changes — hit Save to apply.</span>
                    <else>
                      <for as="p" of={(col[0] === 'allow' ? pAllow() : col[0] === 'ask' ? pAsk() : pDeny())} key={p}>
                        <span>
                          <span>{p}</span>
                          <button type="button" aria-label="Remove" on:click={() => removePerm(col[0], p)}>×</button>
                        </span>
                      <else>
                        <span>none</span>
                      </else></for>
                    </else></if>
                  </div>
                </div>
              </for>
            </div>
            <p>Tool patterns: <code>Read</code>, <code>Write</code>, <code>Bash(git *)</code>, <code>mcp__ship__*</code>. Globs allowed.</p>
          </div>
        </card>

        <card padding="none">
          <div slot="header">
            <h2><icon name="file"></icon>Rules<span>{(agent().rules ?? []).length} rules</span></h2>
            <button variant="outline" size="sm" on:click={() => openPicker('rule')}><icon slot="leading" name="plus"></icon>Add</button>
          </div>
          <div>
            <if cond={(agent().rules ?? []).length === 0}>
              <div>No rules configured. This agent uses project defaults.</div>
            <else>
              <for as="r" of={(agent().rules ?? [])} key={r}>
                <div>
                  <div>
                    <span>{r}</span>
                    <span>Project rule</span>
                  </div>
                </div>
              </for>
            </else></if>
          </div>
        </card>

        <card padding="none">
          <div slot="header">
            <h2>
              <icon><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="4" y1="21" x2="4" y2="14"/><line x1="4" y1="10" x2="4" y2="3"/><line x1="12" y1="21" x2="12" y2="12"/><line x1="12" y1="8" x2="12" y2="3"/><line x1="20" y1="21" x2="20" y2="16"/><line x1="20" y1="12" x2="20" y2="3"/><line x1="2" y1="14" x2="6" y2="14"/><line x1="10" y1="8" x2="14" y2="8"/><line x1="18" y1="16" x2="22" y2="16"/></svg></icon>
              Providers<span>{providers().length} providers</span>
            </h2>
          </div>
          <div>
            <if cond={providers().length === 0}>
              <div>No providers configured. Add one under <code style="font-family:var(--font-mono);font-size:var(--text-xs)">agent.providers</code> to run this agent.</div>
            <else>
              <div>
                <for as="p" of={providers()} key={p.id}>
                  <div style="cursor:pointer;background:var(--card);border-color:var(--border-subtle);box-shadow:var(--shadow-card-rest);display:flex;flex-direction:column">
                    <div>
                      <span>{p.id.slice(0,2)}</span>
                      <span>{p.label}
                        <if cond={p.isDefault}><badge tone="accent" variant="soft" size="sm">default</badge></if>
                      </span>
                    </div>
                    <div><status-dot tone="success" size="sm"></status-dot>defaults</div>
                  </div>
                </for>
              </div>
            </else></if>
          </div>
        </card>

      </column>
      <div></div>
    </div>
  </div>

  <dialog open={pickerOpen()} size="md" on:sail-close={closePicker}>
    <span slot="header">Add {pickerKind() === 'skill' ? 'skill' : pickerKind() === 'mcp' ? 'MCP server' : 'rule'} to <strong>{agent().name}</strong></span>
    <div>
      <if cond={(catalogs()?.[pickerKind()] ?? []).length === 0}>
        <empty-state variant="placeholder" size="sm">
          <span slot="title">No {pickerKind()}s to add</span>
        </empty-state>
      <else>
        <column>
          <for as="entry" of={(catalogs()?.[pickerKind()] ?? [])} key={entry.id}>
            <label>
              <input type="checkbox" checked={pickerSelected().includes(entry.id)} on:change={() => togglePick(entry.id)} />
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
      <button tone="primary" variant="solid" size="sm" disabled={pickerSelected().length === 0} on:click={savePicker}>Add</button>
    </div>
  </dialog>
</if>
</if>
  </main>
</div>
