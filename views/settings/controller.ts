import { controller, signal } from '@ship/sui-runtime'
import type { LiveState } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: () => ({
    advancedOpen: signal(false),
    $mutation: signal<LiveState<unknown>>({ kind: 'ok', value: undefined }),
  }),
})
