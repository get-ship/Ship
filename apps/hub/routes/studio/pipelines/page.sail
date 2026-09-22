<controller>
// Pipelines list (`/studio/pipelines`). Hub port of
// apps/shell/routes/studio/pipelines/controller.ts.
//
// The shell read the deprecated live op `ship.pipelines.summaries` (a GUI
// view-model stream with NO registered mux resolver → mux/subscribe 404 →
// empty/error). This reads the TYPED unary Connect rpcs through the ambient
// `ops` client (host-owned, injected as `clients.ops`):
//
//   - `pipelines.summaries` → PipelineRunSummariesResponse { runs } — run-level
//     projection rows (pipelineRunId/slug/status/startedAt/endedAt/outcome),
//     newest first. Drives the "Active runs" section (non-terminal rows).
//   - `pipelines.list` → PipelinesIndexResponse { items: IndexEntry{id,scope} } —
//     the pipeline DEFINITIONS index. Drives the "Pipelines" section.
//
// Both wrapped in `live(() => …)` so the template's `` block
// drives loading/error/ok. The TYPED contract DELIBERATELY EXCLUDES the GUI
// view-model tokens the shell stream carried (def step_count/total_runs/
// success_rate, run agent/branch/provider/model, providers list, workspace_id,
// stats_subtitle) — the frontend is adapted to what the projection supplies.
//
// Mutations are unary Connect writes off `ops`: `dispatch` (run a pipeline),
// `put` (create a definition), `cancel` (stop a run — the proper verb, replacing
// the shell's best-effort `endTurn`). No `focus` detail route exists on the typed
// contract, so a definition card opens the Dispatch dialog pre-filled instead of
// navigating to a (would-be 404) focus view.
import { controller, signal, derived, live, unwrap } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'

const MINUTE = 60_000
const HOUR = 3_600_000
const DAY = 86_400_000

function formatRelative(iso: string, now = Date.now()): string {
  const t = Date.parse(iso)
  if (Number.isNaN(t)) return iso
  const diff = now - t
  const abs = Math.abs(diff)
  const suffix = diff >= 0 ? 'ago' : 'from now'
  if (abs < MINUTE) return 'just now'
  if (abs < HOUR) return `${Math.floor(abs / MINUTE)}m ${suffix}`
  if (abs < DAY) return `${Math.floor(abs / HOUR)}h ${suffix}`
  const days = Math.floor(abs / DAY)
  if (days < 30) return `${days}d ${suffix}`
  if (days < 365) return `${Math.floor(days / 30)}mo ${suffix}`
  return `${Math.floor(days / 365)}y ${suffix}`
}

// Run lifecycle status (snake_case, from the projection) → the sui-pipeline-run
// state enum (ok|running|awaiting|err|none) and the sui-badge tone.
const RUN_STATE: Record<string, string> = {
  pending: 'running', dispatched: 'running', running: 'running',
  gate_pending: 'awaiting', blocked: 'awaiting',
  completed: 'ok', merged: 'ok', failed: 'err', cancelled: 'none',
}
const RUN_TONE: Record<string, string> = {
  pending: 'info', dispatched: 'info', running: 'info',
  gate_pending: 'warning', blocked: 'warning',
  completed: 'success', merged: 'success', failed: 'destructive', cancelled: 'muted',
}
const TERMINAL = new Set(['completed', 'merged', 'failed', 'cancelled'])

export default controller({
  scope: (ctx: ViewScope<Scope>) => {
    const ops = ctx.ops
    // Primary `{#async}` signal — the run-level projection (newest first).
    const summaries = live(() => ops.pipelines.summaries({}))
    // Secondary — the definitions index. Read via unwrap; degrades to empty
    // while loading / on error without blocking the page.
    const defsData = live(() => ops.pipelines.list({}))

    const activeRuns = derived(() => {
      const rows = unwrap(summaries)?.runs ?? []
      return rows
        .filter((r) => !TERMINAL.has(r.status))
        .map((r) => ({
          slug: r.slug,
          startedAt: r.startedAt,
          startedLabel: r.startedAt ? formatRelative(r.startedAt) : '',
          stateAttr: RUN_STATE[r.status] ?? 'none',
          tone: RUN_TONE[r.status] ?? 'muted',
          statusLabel: r.status.replace(/_/g, ' '),
        }))
    })
    const defs = derived(() => {
      const items = unwrap(defsData)?.items ?? []
      return items.map((i) => ({ name: i.id, scope: i.scope }))
    })
    const statsSubtitle = derived(() => {
      const d = defs().length
      const a = activeRuns().length
      return `${d} pipeline${d === 1 ? '' : 's'} · ${a} active run${a === 1 ? '' : 's'}`
    })

    const dispatchOpen = signal(false)
    const dispatchPipeline = signal('')
    const dispatchSpec = signal('')
    const dispatchHeadless = signal(true)
    const newPipeOpen = signal(false)
    const newPipeName = signal('')
    const newPipeError = signal('')
    const newPipeCreating = signal(false)

    return {
      summaries, activeRuns, defs, statsSubtitle,
      dispatchOpen, dispatchPipeline, dispatchSpec, dispatchHeadless,
      newPipeOpen, newPipeName, newPipeError, newPipeCreating,
    }
  },
  on: {
    openDispatch: (s) => {
      if (!s.dispatchPipeline && s.defs.length > 0) s.dispatchPipeline = s.defs[0].name
      s.dispatchOpen = true
    },
    openDispatchFor: (s, name: string) => {
      s.dispatchPipeline = name
      s.dispatchOpen = true
    },
    closeDispatch: (s) => { s.dispatchOpen = false },
    dispatchRun: async (s) => {
      const name = String(s.dispatchPipeline)
      if (!name) return
      // PipelinesService.Dispatch (unary). harness empty → the pipeline's
      // declared providers; the typed contract exposes no enumerable provider
      // list, so there is no provider-override field.
      await s.clients.ops.pipelines.dispatch({
        pipeline: name,
        spec: String(s.dispatchSpec),
        harness: '',
        auto: Boolean(s.dispatchHeadless),
      })
      s.dispatchOpen = false
    },
    openNewPipe: (s) => { s.newPipeOpen = true; s.newPipeName = ''; s.newPipeError = '' },
    closeNewPipe: (s) => { s.newPipeOpen = false },
    createPipeline: async (s) => {
      const name = String(s.newPipeName).trim()
      if (!/^[a-z0-9][a-z0-9-]*$/.test(name)) {
        s.newPipeError = 'Name must be lowercase kebab-case (a-z, 0-9, -).'; return
      }
      s.newPipeCreating = true; s.newPipeError = ''
      const content = `{\n  "name": "${name}",\n  "description": "",\n  "steps": [\n    { "id": "one", "run": "echo hello" }\n  ]\n}\n`
      // PipelinesService.Put (unary) → writes to the active project, resolved
      // server-side. Reload so the new definition appears in the index.
      try {
        await s.clients.ops.pipelines.put({ name, docJson: content, scope: 'project' })
        window.location.reload()
      } catch (err) { s.newPipeError = (err as Error)?.message ?? String(err) }
      finally { s.newPipeCreating = false }
    },
    cancelRun: async (s, slug: string) => {
      // PipelinesService.Cancel (unary, slug-keyed) — emits PIPELINE_RUN_CANCELLED.
      // The correct stop verb (the shell routed to EndTurn, a likely no-op).
      try { await s.clients.ops.pipelines.cancel({ slug, reason: '' }) } catch { /* best-effort */ }
    },
  },
})
</controller>

<div data-test="studio-view">
  <ship-studio-rail></ship-studio-rail>
  <main data-test="studio-content">

  <while-loading of="summaries">
  <div data-test="loading" role="status" aria-busy="true" aria-label="Loading pipelines">
    <column gap="4">
      <spinner shape="line" size="lg" style="width:14rem"></spinner>
      <div aria-hidden="true">
        <for as="n" of={[0,1,2,3,4,5]} key={n}>
          <spinner shape="block" size="lg" style="height:6rem"></spinner>
        </for>
      </div>
    </column>
  </div>

  </while-loading><while-error as="err" of="summaries">
  <div>
    <error-state title="Couldn't load pipelines" data-test="error" role="alert">
      {err.message}
      <button slot="action" tone="primary" variant="solid" on:click={() => window.location.reload()}>Retry</button>
    </error-state>
  </div>

  </while-error><if cond={summaries().kind === 'ok'}>
<div>
  <div>
    <header>
      <div>
        <h1>Pipelines</h1>
        <p>{statsSubtitle()}</p>
      </div>
      <row gap="2">
        <button variant="outline" size="sm" on:click={() => navigate('/studio/pipelines/history')}>
          <icon slot="leading" name="list" size="sm"></icon>History
        </button>
        <button variant="outline" size="sm" on:click={openNewPipe}>
          <icon slot="leading" name="plus" size="sm"></icon>New pipeline
        </button>
        <button tone="primary" variant="solid" size="sm" on:click={openDispatch}>
          <icon slot="leading" name="play" size="sm"></icon>Dispatch
        </button>
      </row>
    </header>

    <if cond={activeRuns().length > 0}>
    <div>
      <div>
        <h2>
          <icon name="play" size="sm" tone="info"></icon>Active runs
        </h2>
        <badge tone="info">{activeRuns().length}</badge>
      </div>
      <column gap="3">
        <for as="run" of={activeRuns()} key={run.slug}>
          <sui-pipeline-run state={run.stateAttr}>
            {run.slug}
            <span slot="meta">
              <badge tone={run.tone} size="sm">{run.statusLabel}</badge>
              <time datetime={run.startedAt}>{run.startedLabel}</time>
              <button variant="ghost" size="sm" on:click={() => navigate('/studio/pipelines/run/' + run.slug)}>
                <icon slot="leading" name="chevron-right"></icon>Open
              </button>
              <button variant="outline" size="sm" tone="destructive" on:click={() => cancelRun(run.slug)}>
                <icon slot="leading" name="stop"></icon>Cancel
              </button>
            </span>
          </sui-pipeline-run>
        </for>
      </column>
    </div>
    </if>

    <div>
      <div>
        <h2>
          <icon name="git-merge" size="sm"></icon>Pipelines
        </h2>
        <badge tone="muted">{defs().length}</badge>
      </div>
      <if cond={defs().length > 0}>
      <div>
        <for as="def" of={defs()} key={def.name}>
          <sui-pipeline-card interactive on:click={() => openDispatchFor(def.name)}>
            {def.name}
            <span slot="steps">
              <chip size="sm" tone="muted">{def.scope}</chip>
            </span>
          </sui-pipeline-card>
        </for>
      </div>
      <else>
      <empty-state icon="git-merge" heading="No pipelines" description="Define a pipeline in .ship/pipelines/ to see it here."></empty-state>
      </else></if>
    </div>
  </div>

  <div>
    <icon name="monitor" size="xl" tone="muted"></icon>
    <span>Best experienced on desktop</span>
  </div>

  <dialog open={dispatchOpen()} size="lg" on:sail-close={closeDispatch}>
    <span slot="header">Dispatch pipeline run</span>
    <field label="Pipeline">
      <select bind:value={dispatchPipeline}>
        <for as="def" of={defs()} key={def.name}>
          <option value={def.name}>{def.name}</option>
        </for>
      </select>
    </field>
    <field label="Spec" hint="Plain text or markdown — what the run should do.">
      <textarea bind:value={dispatchSpec} rows="10" placeholder="Describe the work in one or two paragraphs..."></textarea>
    </field>
    <field><input type="checkbox" bind:checked={dispatchHeadless}>Headless (run without tmux)</field>
    <div slot="footer">
      <button variant="outline" on:click={closeDispatch}>Cancel</button>
      <button tone="primary" variant="solid" on:click={dispatchRun}>
        <icon slot="leading" name="play" size="sm"></icon>Dispatch
      </button>
    </div>
  </dialog>

  <dialog open={newPipeOpen()} size="sm" on:sail-close={closeNewPipe}>
    <span slot="header">New pipeline</span>
    <field label="Name" hint="Lowercase, kebab-case. Becomes the filename.">
      <input bind:value={newPipeName} placeholder="my-pipeline" autofocus>
    </field>
    <if cond={newPipeError()}><alert tone="destructive" compact>{newPipeError()}</alert></if>
    <div slot="footer">
      <button variant="outline" on:click={closeNewPipe}>Cancel</button>
      <button tone="primary" variant="solid" loading={newPipeCreating()} disabled={newPipeCreating()} on:click={createPipeline}>
        <icon slot="leading" name="plus" size="sm"></icon>Create
      </button>
    </div>
  </dialog>
</div>
</if>
  </main>
</div>
