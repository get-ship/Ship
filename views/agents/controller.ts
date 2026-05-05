import { controller, unwrap } from '@ship/ui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  derived: {
    items: ({ agents }) => unwrap(agents)?.items ?? [],
    total: ({ agents }) => unwrap(agents)?.items.length ?? 0,
  },
  on: {
    newAgent: ({ navigate }) => navigate('/agents/create'),
  },
})
