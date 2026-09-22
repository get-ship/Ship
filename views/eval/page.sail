<controller>
// views/eval — read sui-eval-runs collection rows, group by run_id.
// The runs signal is the live source (declared here, not in <data>,
// because `.items({...})` isn't in the renderable subset and the
// SSR pass refuses to emit it from a <data source=…>).

import { controller, derived, live, unwrap } from '@ship/sail-runtime'
import type { LiveSource, ViewScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'

interface LaneRow {
  run_id: string
  lane_id: string
  model?: string
  composite: number
  compile_passed: boolean
  compile_label: string
  lint_errors: number
  lint_warnings: number
  /** "passed/total" when the run had a behavior spec, else an em dash. */
  behavior_label: string
}

interface RunGroup {
  runId: string
  lanes: LaneRow[]
}

interface CollectionItemsResponse {
  items: Array<{ id: string; data?: Record<string, unknown> }>
}

export default controller({
  scope: (ctx: ViewScope) => {
    const { ship } = ctx
    const runs = live<CollectionItemsResponse>(() =>
      ship.collections.items({ collection: 'sui-eval-runs' }) as unknown as LiveSource<CollectionItemsResponse>
    )

    // Deriveds live in the scope factory — the resolver only merges what this
    // factory returns. (This view used the retired top-level `derived:`
    // section and silently rendered no rows; caught by page.behavior.jsonc.)
    const groups = derived((): RunGroup[] => {
      const items = unwrap(runs)?.items ?? []
      const byRun = new Map<string, LaneRow[]>()
      for (const item of items) {
        const data = (item.data ?? {}) as Partial<LaneRow>
        if (!data.run_id || !data.lane_id) continue
        const compile_passed = data.compile_passed === true
        const raw = (item.data ?? {}) as Record<string, unknown>
        const behaviorTotal = typeof raw.behavior_total === 'number' ? raw.behavior_total : 0
        const behaviorPassed = typeof raw.behavior_passed === 'number' ? raw.behavior_passed : 0
        // exactOptionalPropertyTypes: omit `model` when the row doesn't
        // supply one (the interface declares `model?: string`, not
        // `model?: string | undefined`).
        const lane: LaneRow = data.model !== undefined
          ? { run_id: data.run_id, lane_id: data.lane_id, model: data.model, composite: typeof data.composite === 'number' ? data.composite : 0, compile_passed, compile_label: compile_passed ? 'pass' : 'fail', lint_errors: typeof data.lint_errors === 'number' ? data.lint_errors : 0, lint_warnings: typeof data.lint_warnings === 'number' ? data.lint_warnings : 0, behavior_label: behaviorTotal > 0 ? `${behaviorPassed}/${behaviorTotal}` : '—' }
          : { run_id: data.run_id, lane_id: data.lane_id, composite: typeof data.composite === 'number' ? data.composite : 0, compile_passed, compile_label: compile_passed ? 'pass' : 'fail', lint_errors: typeof data.lint_errors === 'number' ? data.lint_errors : 0, lint_warnings: typeof data.lint_warnings === 'number' ? data.lint_warnings : 0, behavior_label: behaviorTotal > 0 ? `${behaviorPassed}/${behaviorTotal}` : '—' }
        const arr = byRun.get(lane.run_id) ?? []
        arr.push(lane)
        byRun.set(lane.run_id, arr)
      }
      return Array.from(byRun.entries())
        .map(([runId, lanes]) => ({
          runId,
          lanes: lanes.sort((a, b) => a.lane_id.localeCompare(b.lane_id)),
        }))
        .sort((a, b) => b.runId.localeCompare(a.runId))
    })

    return { runs, groups }
  },
})
</controller>

<!--
  views/eval — SUI eval surface. Reads the sui-eval-runs collection
  (one row per (run_id, lane_id)), groups rows by run, lays out a
  side-by-side comparison per run.

  Schema lives in .ship/collections/sui-eval-runs.jsonc. Rows are
  written by .ship/skills/sui-eval/scripts/record.ts via
  `ship ops call collections.create_item`. The runs() live source is
  declared in controller.ts (renderable subset doesn't admit
  `.items()` method calls in <data> source expressions).
-->

<div>
  <header>
    <div>
      <h1>Evals</h1>
      <p>SKILL × model authoring runs from the sui-eval skill.</p>
    </div>
  </header>

  <while-loading of="runs">
    <div role="status" aria-live="polite" aria-busy="true">
      <spinner shape="line" size="lg" style="width:60%"></spinner>
      <spinner shape="line" size="md" style="width:80%"></spinner>
      <spinner shape="line" size="md" style="width:40%"></spinner>
    </div>
  </while-loading>

  <while-error as="err" of="runs">
    <error-state title="Couldn't load eval runs" role="alert">
      {err.message}
    </error-state>
  </while-error>

  <if cond={runs().kind === 'ok'}>
    <if cond={groups().length === 0}>
      <empty-state variant="default" size="md" role="status">
        <icon slot="icon" name="bar-chart-3"></icon>
        <span slot="title">No eval runs yet</span>
        <span slot="description">
          Run the <span>sui-eval</span> skill (dispatch, lane-watch,
          grade, record) to produce the first row. The grade step writes one row per
          (run_id, lane_id) into the <span>sui-eval-runs</span> collection.
        </span>
      </empty-state>
    <else>
      <ul aria-label="Eval runs">
        <for as="grp" of={groups()} key={grp.runId}>
          <li>
            <div>
              <span>{grp.runId}</span>
              <span>{grp.lanes.length} lane{grp.lanes.length === 1 ? '' : 's'}</span>
            </div>
            <div>
              <for as="lane" of={grp.lanes} key={lane.lane_id}>
                <article data-passed={lane.compile_passed ? 'true' : 'false'}>
                  <header>
                    <span>{lane.lane_id}</span>
                    <if cond={lane.model}><span>{lane.model}</span></if>
                  </header>
                  <div>
                    <span>{lane.composite.toFixed(2)}</span>
                    <span>composite</span>
                  </div>
                  <dl>
                    <div>
                      <dt>compile</dt>
                      <dd>{lane.compile_label}</dd>
                    </div>
                    <div>
                      <dt>lint errors</dt>
                      <dd>{lane.lint_errors}</dd>
                    </div>
                    <div>
                      <dt>lint warnings</dt>
                      <dd>{lane.lint_warnings}</dd>
                    </div>
                    <div>
                      <dt>behavior</dt>
                      <dd>{lane.behavior_label}</dd>
                    </div>
                  </dl>
                </article>
              </for>
            </div>
          </li>
        </for>
      </ul>
    </else></if>
  </if>
</div>
