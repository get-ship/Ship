// <sui-pipeline-trigger> — derives the kind label so the template renders it
// as real DOM text. The glyph stays a decorative ::before symbol.
import { controller, derived } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-pipeline-trigger.sui'

export default controller<Scope>({
  scope: ({ kind }: ViewScope<Scope>) => {
    const kindLabel = derived(() => kind())
    return { kindLabel }
  },
})
