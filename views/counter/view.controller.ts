import { controller, signal } from '@ship/ui-runtime'
import type { Scope } from './view.sui'

export default controller<Scope>({
  scope: () => ({ count: signal(0) }),
})
