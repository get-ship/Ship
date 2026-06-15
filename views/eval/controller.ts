// views/eval — read sui-eval-runs collection rows, group by run_id.
// The runs signal is the live source (declared here, not in <data>,
// because `.items({...})` isn't in the renderable subset and the
// SSR pass refuses to emit it from a <data source=…>).

import { controller, derived, live, unwrap } from '@ship/sui-runtime'
import type { LiveSource, ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

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

export default controller<Scope>({
  scope: (ctx: ViewScope<Scope>) => {
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
