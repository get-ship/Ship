import { controller, signal } from '@ship/ui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: () => ({ count: signal(0) }),
})
