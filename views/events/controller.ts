import { controller, signal, derived, live, unwrap } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: ({ ship }) => {
    const q = signal('')
    const eventType = signal('')
    const actor = signal('')
    const entity = signal('')
    const eventsData = live(() => ship.events.list({
      q: q(),
      event_type: eventType(),
      actor: actor(),
      entity: entity(),
    }))
    const totalCount = derived(() => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const d = unwrap(eventsData as any)
      if (!d) return 0
      return (d.groups as { rows: unknown[] }[]).reduce((acc, g) => acc + g.rows.length, 0)
    })
    return { q, eventType, actor, entity, eventsData, totalCount }
  },
})
