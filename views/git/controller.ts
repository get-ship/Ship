// Git view controller — derives every UI surface from a single
// `overview` live signal subscribed via the daemon's `_sui/data` SSE.
//
// View-host (`/sui/<name>`) mounts views without a `ship` SDK client in
// ctx, so the compiler-emitted `<data live>` path (which closes over
// `ctx.ship`) sticks on `loading`. `liveExpr` opens the `/_sui/data`
// SSE channel directly — the sanctioned way for view-host controllers.

import { controller, signal, live, unwrapOr, useFilteredList } from '@ship/sui-runtime'
import { liveExpr } from '@ship/sdk-client'
import type { GitOverview, GitCommit } from '@ship/sdk-client'
import type { Scope } from './page.sui'

type GitTab = 'changes' | 'history' | 'branches'

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

export default controller<Scope>({
  scope: () => ({
    tab: signal<GitTab>('changes'),
    overview: live(() => liveExpr<GitOverview>('ship.git.overview()')),
  }),

  derived: {
    branch:         (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).branch,
    headShort:      (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).head_short,
    clean:          (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).clean,
    totalChanged:   (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).total_changed,
    ahead:          (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).ahead,
    behind:         (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).behind,
    staged:         (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).staged,
    modified:       (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).modified,
    untracked:      (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).untracked,
    commits:        (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).commits,
    branches:       (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).branches,
    commitsCount:   (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).commits.length,
    branchesCount:  (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).branches.length,
    recentCommits:  (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).commits.slice(0, 5),
    recentBranches: (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).branches.slice(0, 6),
  },

  use: (s) => ({
    commitFilter: useFilteredList<GitCommit>(() => unwrapOr(s.overview, EMPTY_OVERVIEW).commits, (c, q) => {
      if (!q) return true
      const needle = q.toLowerCase()
      return (
        c.subject.toLowerCase().includes(needle) ||
        c.short_sha.toLowerCase().includes(needle) ||
        c.author.toLowerCase().includes(needle)
      )
    }),
  }),

  on: {
    showChanges:  (s) => { s.tab = 'changes' },
    showHistory:  (s) => { s.tab = 'history' },
    showBranches: (s) => { s.tab = 'branches' },
  },
})
