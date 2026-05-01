// Git view controller — derives every UI surface from a single
// `<data live name="overview" />` subscription.
//
// The j2 prod view (`.ship/views/git/index.html.j2`) ALSO calls
// `ship.git.diff(commit=...)` and `ship.git.annotations(file=...)`
// when query params are set, to render the split-pane diff. Those
// calls take arguments and the `_ship-set/data` SSE endpoint doesn't
// yet support arg-bearing live expressions — so this port covers
// every panel driven by `git.overview()` (changes-clean hero, file
// lists, history, branches) and skips the diff pane for now.
//
// Stress-tests: a controller hook (`useFilteredList` over commits),
// a template hook (`<use-kbd>` for tab keyboard shortcuts), envelope
// unwrap via `unwrapOr`, signal-passing into the hook factory.

import { controller, unwrapOr, useFilteredList } from '@ship/ui-runtime'
import type { LiveSignal, Signal } from '@ship/ui-runtime'
import type { GitOverview, GitCommit } from '@ship/ui-sdk-client'

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

/** Template scope — every identifier the .sui file declares (via
 * `<data>`, `<state>`, `<use-* as=>`) that the controller wants to
 * read. Pass as the `controller<Scope>` generic and the sections'
 * destructure args type themselves; no per-section annotations
 * needed. State (`tab`) lives in the template via `<state>` per the
 * declarative-state ADR. */
type Scope = {
  overview: LiveSignal<GitOverview>
  tab: Signal<'changes' | 'history' | 'branches'>
}

/** Single read of the live envelope; reused by every overview-derived. */
const read = ({ overview }: Scope) => unwrapOr(overview, EMPTY_OVERVIEW)

export default controller<Scope>({
  derived: {
    branch:         (s) => read(s).branch,
    headShort:      (s) => read(s).head_short,
    clean:          (s) => read(s).clean,
    totalChanged:   (s) => read(s).total_changed,
    ahead:          (s) => read(s).ahead,
    behind:         (s) => read(s).behind,
    staged:         (s) => read(s).staged,
    modified:       (s) => read(s).modified,
    untracked:      (s) => read(s).untracked,
    commits:        (s) => read(s).commits,
    branches:       (s) => read(s).branches,
    commitsCount:   (s) => read(s).commits.length,
    branchesCount:  (s) => read(s).branches.length,
    recentCommits:  (s) => read(s).commits.slice(0, 5),
    recentBranches: (s) => read(s).branches.slice(0, 6),
  },

  use: (s) => ({
    // Filter on subject, short sha, or author — three needles, one query.
    commitFilter: useFilteredList<GitCommit>(() => read(s).commits, (c, q) => {
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
