// Skills view controller — owns the rail, the selected-skill detail,
// recent invocation rows for the Runs tab, and the invoke mutation.
//
// View-host (`/sui/<name>`) mounts views without a `ship` SDK client in
// ctx, so reaching the daemon via `ctx.ship.skills.…` sticks on
// `loading`. Wire the live sources directly to `/_sui/data?expr=…`
// instead. `skill` / `runs` only fire once a skillId is selected — both
// expressions short-circuit to `__none__` until then so the daemon
// returns empty results instead of erroring. `invoke` posts to the REST
// mutation endpoint and writes the parsed response (run_id +
// session_id) or the error into `invokeResult` for the inline banner.
//
// The Source button toggles an in-pane raw view of SKILL.md. Wiring
// SKILL.md into the dock's Source tab is a separate concern documented
// in `.artifacts/sui-arc/skills-detail-pane.md` — the dock currently
// hard-binds to the active view's template path and refactoring that
// is out of scope here.

import { controller, signal, live, type LiveSignal, type LiveSource } from '@ship/sui-runtime'
import { liveExpr } from '@ship/sdk-client'
import type { Scope } from './page.sui'

interface EventRow {
  id: string
  event_type: string
  entity_id: string
  actor: string
  workspace_id: string
  session_id: string
  timestamp: string
  payload_pretty: string
}

interface EventsValue {
  groups: Array<{ entity_id: string; rows: EventRow[] }>
}

interface InvokeOk {
  ok: true
  run_id: string
  session_id: string
  workspace_id: string | null
  mode: string
}

interface InvokeErr {
  ok: false
  error: string
}

type InvokeResult = InvokeOk | InvokeErr

// `runsSource` is a specialized source — it short-circuits to an empty
// result when no skill is selected and flattens the daemon's grouped
// `events.list` payload to a flat row list. The generic live channels
// (`rail`, `skill`) use `liveExpr` from the SDK directly.
function runsSource(skillId: string): LiveSource<{ rows: EventRow[] }> {
  const expr = `ship.events.list(${JSON.stringify({
    event_type: 'skill.started',
    entity: skillId || '__none__',
    limit: 20,
  })})`
  return {
    subscribe(sig: LiveSignal<{ rows: EventRow[] }>): () => void {
      if (!skillId) {
        sig.set({ kind: 'ok', value: { rows: [] } })
        return () => {}
      }
      const es = new EventSource(`/_sui/data?expr=${encodeURIComponent(expr)}`)
      const onPatch = (ev: MessageEvent): void => {
        try {
          const patch = JSON.parse(ev.data as string) as { op: string; value: EventsValue }
          if (patch.op === 'set') {
            const groups = patch.value?.groups ?? []
            const rows = groups.flatMap((g) => g.rows ?? [])
            sig.set({ kind: 'ok', value: { rows } })
          }
        } catch (err) {
          sig.set({ kind: 'error', error: err instanceof Error ? err : new Error(String(err)) })
        }
      }
      es.addEventListener('patch', onPatch as EventListener)
      return () => { es.close() }
    },
  }
}

function skillsExpr(args: { q?: string; kind?: string; tag?: string }): string {
  const filtered: Record<string, string> = {}
  for (const [k, v] of Object.entries(args)) {
    if (typeof v === 'string' && v.length > 0 && v !== 'all') filtered[k] = v
  }
  const lit = Object.keys(filtered).length === 0 ? '' : JSON.stringify(filtered)
  return `ship.skills.summaries(${lit})`
}

export default controller<Scope>({
  scope: () => {
    const q = signal('')
    const kind = signal('all')
    const tag = signal('')
    const skillId = signal('')
    const tab = signal('content')
    const viewRaw = signal(false)
    const invokeBusy = signal<string>('')
    const invokeResult = signal<InvokeResult | null>(null)
    const rail = live(() => liveExpr<unknown>(skillsExpr({ q: q(), kind: kind(), tag: tag() })))
    const skill = live(() => liveExpr<unknown>(
      `ship.skills.detail(${JSON.stringify({ id: skillId() || '__none__' })})`,
    ))
    const runs = live(() => runsSource(skillId()))

    function openSource(): void {
      viewRaw.set(!viewRaw())
      if (viewRaw()) tab.set('content')
    }

    async function invoke(id: string): Promise<void> {
      invokeBusy.set(id)
      invokeResult.set(null)
      try {
        const resp = await fetch(`/api/v1/skills/${encodeURIComponent(id)}/invoke`, {
          method: 'POST',
          credentials: 'same-origin',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify({}),
        })
        const text = await resp.text().catch(() => '')
        if (!resp.ok) {
          let msg = text || `Skill invoke failed (${resp.status})`
          if (resp.status === 404 || resp.status === 501 || text.includes('not_implemented')) {
            msg = 'Skill runs are coming soon — server route not wired yet.'
          }
          invokeResult.set({ ok: false, error: msg })
          return
        }
        const body = (text ? JSON.parse(text) : {}) as Partial<InvokeOk> & { ok?: boolean }
        if (body.ok && typeof body.run_id === 'string' && typeof body.session_id === 'string') {
          invokeResult.set({
            ok: true,
            run_id: body.run_id,
            session_id: body.session_id,
            workspace_id: body.workspace_id ?? null,
            mode: body.mode ?? 'library',
          })
        } else {
          invokeResult.set({ ok: false, error: 'Skill invoke returned an unexpected response.' })
        }
      } catch (e) {
        invokeResult.set({ ok: false, error: (e as Error).message ?? 'Skill invoke failed.' })
      } finally {
        invokeBusy.set('')
      }
    }

    return {
      q, kind, tag, skillId, tab, viewRaw,
      rail, skill, runs,
      invoke, invokeBusy, invokeResult, openSource,
    }
  },
})
