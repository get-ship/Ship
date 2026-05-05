import { controller, signal } from '@ship/ui-runtime'
import type { Scope } from './without-island.sui'

export default controller<Scope>({
  scope: () => ({ count: signal(0) }),
})
