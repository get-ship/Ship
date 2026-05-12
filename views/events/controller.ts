import { controller, signal, derived, unwrap } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: ({ eventsData }) => {
    const q = signal('')
    const eventType = signal('')
    const actor = signal('')
    const entity = signal('')
    const totalCount = derived(() => {
      const d = unwrap(eventsData as any)
      if (!d) return 0
      return (d.groups as { rows: unknown[] }[]).reduce((acc, g) => acc + g.rows.length, 0)
    })
    return { q, eventType, actor, entity, totalCount }
  },
})
