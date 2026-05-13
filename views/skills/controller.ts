// Skills view controller — owns the `rail` + `skill` live signals.
//
// View-host (`/sui/<name>`) mounts views without a `ship` SDK client in
// ctx, so reaching the daemon via `ctx.ship.skills.…` sticks on
// `loading`. Wire the live sources directly to `/_sui/data?expr=…`
// instead. `skill` only fires once a skillId has been selected — the
// dashboard panel is hidden by `kind === 'ok'` until then, matching
// the previous behavior. `invoke` posts to the existing REST mutation
// endpoint and degrades to a banner when the daemon doesn't have the
// route yet (tracked in daemon-sdk-mutation-ops).

import { controller, signal, live, type LiveSignal, type LiveSource } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

function sseSource<T>(expr: string): LiveSource<T> {
  return {
    subscribe(sig: LiveSignal<T>): () => void {
      const es = new EventSource(`/_sui/data?expr=${encodeURIComponent(expr)}`)
      const onPatch = (ev: MessageEvent): void => {
        try {
          const patch = JSON.parse(ev.data as string) as { op: string; value: T }
          if (patch.op === 'set') sig.set({ kind: 'ok', value: patch.value })
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
    const invokeBusy = signal<string>('')
    const invokeError = signal<string>('')
    const rail = live(() => sseSource<unknown>(skillsExpr({ q: q(), kind: kind(), tag: tag() })))
    const skill = live(() => sseSource<unknown>(
      `ship.skills.detail(${JSON.stringify({ id: skillId() || '__none__' })})`,
    ))

    async function invoke(id: string): Promise<void> {
      invokeBusy.set(id)
      invokeError.set('')
      try {
        const resp = await fetch(`/api/v1/skills/${encodeURIComponent(id)}/invoke`, {
          method: 'POST',
          credentials: 'same-origin',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify({}),
        })
        if (!resp.ok) {
          const text = await resp.text().catch(() => '')
          if (resp.status === 404 || resp.status === 501 || text.includes('not_implemented')) {
            invokeError.set('Skill runs are coming soon — server route not wired yet.')
          } else {
            invokeError.set(text || `Skill invoke failed (${resp.status})`)
          }
        }
      } catch (e) {
        invokeError.set((e as Error).message ?? 'Skill invoke failed.')
      } finally {
        invokeBusy.set('')
      }
    }

    return { q, kind, tag, skillId, rail, skill, invoke, invokeBusy, invokeError }
  },
})
