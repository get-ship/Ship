// Skills index controller — explorer rail only.
//
// The index mirrors the OG j2 shell: a filterable explorer on the left
// and an empty editor pane on the right. Selecting a skill is a
// navigation (`<a href="/skills/:id">`) rather than an in-page tab
// switch — that satisfies the harvest's AVOID rule about list rows
// needing real anchors so right-click "open in new tab" + URL copy work.
//
// We keep an in-row Invoke button on each skill so the explorer can
// kick off a library-skill run without leaving the index. The mutation
// posts to `/api/v1/skills/{id}/invoke` and surfaces the result in an
// inline banner above the empty editor.
//
// `liveExpr` is used directly (rather than `ctx.ship.skills.*`) because
// the view-host (`/sui/<name>`) mounts views without an SDK client in
// ctx; the SSE channel reaches the daemon's data registry directly.

import { controller, signal, live } from '@ship/sui-runtime'
import { liveExpr } from '@ship/sdk-client'
import type { Scope } from './page.sui'

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
    const invokeBusy = signal<string>('')
    const invokeResult = signal<InvokeResult | null>(null)
    const rail = live(() => liveExpr<unknown>(skillsExpr({ q: q(), kind: kind(), tag: tag() })))

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

    return { q, kind, tag, rail, invoke, invokeBusy, invokeResult }
  },
})
