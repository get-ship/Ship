import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: () => {
    const q = signal('')
    const kind = signal('all')
    const tag = signal('')
    return { q, kind, tag }
  },
})
