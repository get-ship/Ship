import { controller, signal, live } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: ({ ship }) => {
    const q = signal('')
    const kind = signal('all')
    const tag = signal('')
    const skillId = signal('')
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const s = ship as any
    const rail = live(() => s.skills.summaries({ q: q(), kind: kind(), tag: tag() }))
    const skill = live(() => s.skills.detail({ id: skillId() }))
    return { q, kind, tag, skillId, rail, skill }
  },
})
