<controller>
import { controller, signal, derived, live, unwrap } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { navigate } from '@ship/deck-router'
import { routes } from '../../../../routes.gen'

type TemplateEntry = {
  id: string
  label: string
  description: string
  letter: string
  hue: string
  skill_ids: string[]
  permission_preset: string
}

type CreateCtxValue = {
  templates: TemplateEntry[]
  permission_presets: Array<{ id: string; label: string; description: string }>
  // The backend serializes the selectable provider/harness list as
  // `available_harnesses` (claude/codex/cursor/opencode).
  available_harnesses: string[]
}

function slugify(input: string): string {
  return input
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/(^-|-$)/g, '')
}

function isValidId(id: string): boolean {
  return /^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(id)
}

export default controller((ctx: ViewScope<Scope>) => {

    // AMBIENT `ops` client. `agents.CreateCtx` returns a `JsonResponse`
    // ({ json }) wrapping the serialized create-wizard payload; `createData`
    // decodes it. `createCtx` stays the LiveSignal that drives `{#async}`.
    const ops = ctx.ops
    const createCtx = live(() => ops.agents.createCtx({}))
    const createData = derived<CreateCtxValue>(() => {
      const empty: CreateCtxValue = { templates: [], permission_presets: [], available_harnesses: [] }
      const r = unwrap(createCtx)
      if (!r) return empty
      try {
        return JSON.parse(r.json) as CreateCtxValue
      } catch {
        return empty
      }
    })
    const step = signal(0)
    const template = signal('blank')
    const templateSkills = signal<string[]>([])
    const hue = signal('orange')
    const name = signal('')
    const id = signal('')
    const desc = signal('')
    const providers = signal<string[]>(['claude'])
    const preset = signal('standard')
    const creating = signal(false)
    const err = signal('')

    // Resolved spec preview shown on the review step.
    const resolvedDoc = derived(() => {
      const doc: Record<string, unknown> = {
        $schema: 'https://raw.githubusercontent.com/madvib/ship/main/schemas/agent.schema.json',
        schema_version: '1',
        agent: {
          id: id(),
          name: name() || id(),
          description: desc(),
          providers: providers(),
          tags: [],
        },
        permissions: { preset: preset() },
      }
      const skills = templateSkills()
      if (skills.length > 0) doc.skills = { refs: skills }
      return doc
    })

    const reviewJson = derived(() => JSON.stringify(resolvedDoc(), null, 2))

    const idValid = derived(() => isValidId(id()))
    const idError = derived(() => {
      const v = id()
      if (!v) return 'ID is required.'
      if (!idValid()) return 'Use lowercase letters, digits, and hyphens (no leading/trailing dash).'
      return ''
    })
    const stepValid = derived(() => {
      switch (step()) {
        case 0: return !!template()
        case 1: return idValid() && !!(name() || id())
        case 2: return providers().length > 0 && !!preset()
        case 3: return idValid()
        default: return false
      }
    })

    
    const cancel = () => navigate(routes.studioAgents())
    const back = (s) => { s.step = Math.max(0, s.step - 1) }
    const next = (s) => {
      if (!s.stepValid) return
      s.step = Math.min(3, s.step + 1)
    }
    const goTo = (n: number) => { if (n <= step) step = n }
    const selectTemplate = (t: TemplateEntry) => {
      template = t.id
      preset = t.permission_preset
      hue = t.hue
      templateSkills = Array.isArray(t.skill_ids) ? t.skill_ids : []
    }
    const toggleProvider = (p: string) => {
      const cur = providers
      providers = cur.includes(p) ? cur.filter((x: string) => x !== p) : [...cur, p]
    }
    const nameInput = (e: CustomEvent<{ value: string }>) => {
      name = e.detail.value
      if (!id && e.detail.value) {
        id = slugify(e.detail.value)
      }
    }
    const idInput = (e: CustomEvent<{ value: string }>) => {
      id = slugify(e.detail.value)
    }
    const descInput = (e: CustomEvent<{ value: string }>) => { desc = e.detail.value }
    const create = async (s) => {
      if (!s.stepValid) return
      s.creating = true
      s.err = ''
      try {
        const m = await s.clients.ops.agents.create({
          id: s.id,
          docJson: JSON.stringify(s.resolvedDoc),
          scope: '',
        })
        navigate(routes.studioAgentsId({ id: m.id }))
        return
      } catch (e) {
        s.err = (e as Error).message ?? 'Network error'
      } finally {
        s.creating = false
      }
    }
    return {
      createCtx, createData, step, template, templateSkills, hue, name, id, desc, providers, preset,
      creating, err,
      resolvedDoc, reviewJson, idValid, idError, stepValid, cancel, back, next, goTo, selectTemplate, toggleProvider, nameInput, idInput, descInput, create }
})

// Re-exported for the Scope type generator.
export type { CreateCtxValue }
</controller>

<div data-test="studio-view">
  <ship-studio-rail></ship-studio-rail>
  <main data-test="studio-content">
<ship-view-chrome title="New agent">
  <button slot="actions" variant="ghost" size="sm" on:click={cancel}>
    <icon slot="leading" name="x" size="xs"></icon>
    Cancel
  </button>
</ship-view-chrome>

<div>

  <div>
    <for as="entry" of={[['Template',0],['Identity',1],['Wiring',2],['Review',3]]} key={entry[1]}>
      <if cond={entry[1] > 0}><span></span></if>
      <button type="button"
        data-active={step() === entry[1]}
        data-done={step() > entry[1]}
        on:click={() => goTo(entry[1])}>
        <span>{step() > entry[1] ? '✓' : entry[1] + 1}</span>
        <span>{entry[0]}</span>
      </button>
    </for>
  </div>

  
    <while-loading of="createCtx">
    <spinner size="md" centered></spinner>

    </while-loading><if cond={createCtx().kind === 'ok'}>

    <if cond={step() === 0}>
      <h2>Start from a template</h2>
      <p>Templates pre-wire skills, MCP servers, and a sensible permission preset. You can change everything in the next steps.</p>
      <div>
        <for as="t" of={createData().templates} key={t.id}>
          <card padding="none"
            data-selected={template() === t.id ? 'true' : null}
            on:click={() => selectTemplate(t)}
            data-test="template"
            data-template-id={t.id}>
            <span>{t.letter}</span>
            <div style="flex:1; min-width:0">
              <div>{t.label}</div>
              <div>{t.description}</div>
              <if cond={t.skill_ids.length > 0}>
                <div>
                  <for as="sId" of={t.skill_ids.slice(0, 4)} key={sId}>
                    <chip tone="muted" variant="outline" size="sm">{sId}</chip>
                  </for>
                  <if cond={t.skill_ids.length > 4}>
                    <chip tone="muted" variant="outline" size="sm">+{t.skill_ids.length - 4}</chip>
                  </if>
                </div>
              </if>
            </div>
          </card>
        </for>
      </div>
    </if>

    <if cond={step() === 1}>
      <h2>Identity</h2>
      <p>A human name plus a machine id. The id matches the filename stem — <code>rust-compiler.jsonc</code> has id <code>rust-compiler</code>.</p>
      <div>
        <div>
          <div style="width:64px; height:64px; font-size:var(--text-2xl)">
            {(id() && id()[0] ? id()[0] : (name() && name()[0] ? name()[0] : '?')).toUpperCase()}
          </div>
          <div>
            <for as="h" of={['orange','amber','red','violet','blue','teal','green','pink','slate']} key={h}>
              <div
                data-selected={hue() === h ? 'true' : null}
                on:click={() => hue.set(h)}
                role="button" tabindex="0" aria-label={h + ' avatar color'}></div>
            </for>
          </div>
        </div>
        <div>
          <div>
            <div>
              <label>Name</label>
              <input value={name()} on:sail-input={nameInput} placeholder="e.g. Rust Compiler" data-test="name-input">
            </div>
            <div>
              <label>ID</label>
              <input value={id()} on:sail-input={idInput} placeholder="rust-compiler"
                invalid={!idValid() && id()}
                error-message={idError()}
                data-test="id-input">
              <if cond={!idValid() && id()}>
                <div data-test="id-error">{idError()}</div>
              </if>
            </div>
          </div>
          <div>saves to <code>.ship/agents/{id() || '(id)'}.jsonc</code></div>
          <div>
            <label>Description</label>
            <textarea value={desc()} on:sail-input={descInput} rows="3" placeholder="What does this agent do?" data-test="desc-input"></textarea>
          </div>
        </div>
      </div>
    </if>

    <if cond={step() === 2}>
      <h2>Wiring</h2>
      <p>Which AI providers this agent compiles to, and what permission preset applies.</p>
      <div>
        <label>Providers</label>
        <div style:grid-template-columns={`repeat(${createData().available_harnesses.length}, 1fr)`}>
          <for as="p" of={createData().available_harnesses} key={p}>
            <card padding="sm"
              data-selected={providers().includes(p) ? 'true' : null}
              on:click={() => toggleProvider(p)}
              style="flex-direction:column; align-items:center; gap:6px; text-align:center">
              <span style="font-size:var(--text-xs); text-transform:capitalize">{p}</span>
            </card>
          </for>
        </div>
      </div>
      <div style="margin-top: var(--space-5)">
        <label>Permission preset</label>
        <div>
          <for as="p" of={createData().permission_presets} key={p.id}>
            <card padding="sm"
              data-selected={preset() === p.id ? 'true' : null}
              on:click={() => preset.set(p.id)}>
              <div style="display:flex; align-items:center; gap:var(--space-3)">
                <div style="flex:1">
                  <div style="font-weight:var(--font-weight-semibold); font-size:var(--text-md)">{p.label}</div>
                  <div style="font-size:var(--text-xs); color:var(--muted-foreground); margin-top:2px">{p.description}</div>
                </div>
              </div>
            </card>
          </for>
        </div>
      </div>
    </if>

    <if cond={step() === 3}>
      <h2>Review</h2>
      <p>Creates <code>.ship/agents/{id() || 'new'}.jsonc</code> on submit.</p>
      <div>
        <div>
          <span>Identity</span>
          <span><strong>{name() || id()}</strong> · <code>{id()}</code></span>
        </div>
        <div>
          <span>Template</span>
          <span>{template()}</span>
        </div>
        <div>
          <span>Providers</span>
          <span>
            <if cond={providers().length === 0}>
              <em>none selected</em>
            <else>
              <for as="p" of={providers()} key={p}>
                <chip tone="primary" variant="soft" size="sm">{p}</chip>
              </for>
            </else></if>
          </span>
        </div>
        <div>
          <span>Preset</span>
          <span><chip tone="muted" variant="soft" size="sm">{preset()}</chip></span>
        </div>
        <if cond={templateSkills().length > 0}>
          <div>
            <span>Skills</span>
            <span>
              <for as="sId" of={templateSkills()} key={sId}>
                <chip tone="muted" variant="outline" size="sm">{sId}</chip>
              </for>
            </span>
          </div>
        </if>
      </div>
      <card padding="md" data-test="review-doc">
        <pre>{reviewJson()}</pre>
      </card>
      <if cond={err()}>
        <alert tone="destructive" style="margin-top:var(--space-3)" data-test="create-err">{err()}</alert>
      </if>
    </if>


  </if>

  <div>
    <if cond={step() > 0}>
      <button variant="ghost" size="sm" on:click={back}>
        <icon slot="leading" name="arrow-left" size="xs"></icon>Back
      </button>
    </if>
    <span></span>
    <button variant="ghost" size="sm" on:click={cancel}>Cancel</button>
    <if cond={step() !== 3}>
      <button tone="primary" variant="solid" size="sm"
        disabled={!stepValid()}
        on:click={next}
        data-test="continue">
        Continue
        <icon slot="trailing" name="arrow-right" size="xs"></icon>
      </button>
    <else>
      <button tone="primary" variant="solid" size="sm"
        disabled={creating() || !stepValid()}
        on:click={create}
        data-test="create">
        <icon slot="leading" name="check" size="xs"></icon>
        {creating() ? 'Creating…' : 'Create agent'}
      </button>
    </else></if>
  </div>

</div>
  </main>
</div>
