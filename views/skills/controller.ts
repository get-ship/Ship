import { controller, signal, live } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: ({ ship }) => {
    const q = signal('')
    const kind = signal('all')
    const tag = signal('')
    const skillId = signal('')
    const invokeBusy = signal<string>('')
    const invokeError = signal<string>('')
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const s = ship as any
    const rail = live(() => s.skills.summaries({ q: q(), kind: kind(), tag: tag() }))
    const skill = live(() => s.skills.detail({ id: skillId() }))

    // ship.skills.invoke() is wired client-side but the daemon route
    // is missing — the SDK rejects with `code: 'not_implemented'`.
    // The view degrades to a banner so the affordance is visible
    // without pretending the run started.
    async function invoke(id: string): Promise<void> {
      invokeBusy.set(id)
      invokeError.set('')
      try {
        await s.skills.invoke({ id })
      } catch (e) {
        const err = e as { code?: string; message?: string }
        invokeError.set(
          err.code === 'not_implemented'
            ? 'Skill runs are coming soon — server route not wired yet.'
            : (err.message ?? 'Skill invoke failed.'),
        )
      } finally {
        invokeBusy.set('')
      }
    }

    return { q, kind, tag, skillId, rail, skill, invoke, invokeBusy, invokeError }
  },
})
