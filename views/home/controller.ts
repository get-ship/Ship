// Home view — proof of the new directory convention plus the
// end-to-end `<data live>` SSE loop. The `home-hero` private widget
// under `_components/` is pure presentation and has no paired
// controller; the router's registry picks it up from `<tag>.sui` and
// the compiler emits its `customElements.define()`. The visit counter
// signal lives in the controller scope (the legacy in-template
// `<state>` form was removed when authoring discipline moved every
// reactive declaration into the controller).
//
// The `git` signal arrives from the template's
// `<data live source="ship.git.overview()" live />` block. The
// compiler emits `live(() => ship.git.overview())`; the SDK opens
// `/_sui/data?expr=ship.git.overview()` and pushes a `patch` event
// every time the daemon's git_signal singleton updates. Derived
// helpers below project the envelope so the template stays terse.

import { controller, signal, unwrapOr } from '@ship/sui-runtime'
import type { GitOverview } from '@ship/sdk-client'
import type { Scope } from './page.sui'

const EMPTY_OVERVIEW: GitOverview = {
  workspace_id: null,
  branch: '',
  head_short: '',
  clean: true,
  total_changed: 0,
  staged: [],
  modified: [],
  untracked: [],
  commits: [],
  branches: [],
  ahead: 0,
  behind: 0,
}

const read = ({ git }: Scope) => unwrapOr(git, EMPTY_OVERVIEW)

export default controller<Scope>({
  scope: () => ({ seen: signal(0) }),

  derived: {
    branch:    (s) => read(s).branch,
    headShort: (s) => read(s).head_short,
    clean:     (s) => read(s).clean,
    changedSummary: (s) => {
      const o = read(s)
      const n = o.total_changed
      return `${n} changed`
    },
  },
})
