import { controller } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-status-bar.sui'

// Glance-only ambient status strip. The only intent is opening the run/session
// view when the operator clicks the sessions or approvals segment. `source`
// tells the host which segment was clicked so it can land on the right view.
export default controller({
  scope: (_props: ViewScope<Scope>, host) => {
    function openRuns(e: Event): void {
      const source = (e.currentTarget as HTMLElement | null)?.classList.contains('approvals')
        ? 'approvals'
        : 'sessions'
      host?.dispatchEvent(
        new CustomEvent('ship-open-runs', {
          detail: { source },
          bubbles: true,
          composed: true,
        }),
      )
    }
    return { openRuns }
  },
})
